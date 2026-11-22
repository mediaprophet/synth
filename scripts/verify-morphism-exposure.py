"""Verify the WP-25 catalogue and exposure gate from saved audit outputs."""

import argparse
import json
from collections import Counter
from pathlib import Path


DARK_BUCKET = "CompletelyDarkAndUncatalogued"


def read_payload(path):
    payload = json.loads(path.read_text(encoding="utf-8-sig"))
    if not isinstance(payload, dict) or payload.get("error"):
        raise ValueError(f"{path} does not contain a successful JSON object")
    payload = payload.get("result", payload)
    if not isinstance(payload, dict) or payload.get("isError"):
        raise ValueError(f"{path} contains an RPC error")
    structured = payload.get("structuredContent")
    if isinstance(structured, dict):
        return structured
    if any(field in payload for field in ("operations", "morphisms", "modules")):
        return payload
    for block in payload.get("content", []):
        if isinstance(block, dict) and block.get("type") == "text":
            structured = json.loads(block["text"])
            if isinstance(structured, dict):
                return structured
    raise ValueError(f"{path} does not contain a structured tool result")


def catalogue_ids(payload, key):
    rows = payload.get(key)
    if not isinstance(rows, list):
        raise ValueError(f"Catalogue is missing the {key} array")
    ids = [row.get("id") if isinstance(row, dict) else row for row in rows]
    if any(not isinstance(item, str) or not item for item in ids):
        raise ValueError(f"The {key} catalogue contains a malformed ID")
    duplicates = sorted(item for item, count in Counter(ids).items() if count > 1)
    if duplicates:
        raise ValueError(f"Duplicate {key} IDs: {', '.join(duplicates)}")
    return set(ids)


def join_catalogue_pages(paths, key):
    rows = []
    for path in paths:
        payload = read_payload(path)
        page = payload.get(key)
        if not isinstance(page, list):
            raise ValueError(f"{path} is missing the {key} array")
        if "offset" in payload and payload["offset"] != len(rows):
            raise ValueError(f"{path} does not begin at offset {len(rows)}")
        if "returned" in payload and payload["returned"] != len(page):
            raise ValueError(f"{path} does not match its declared returned count")
        rows.extend(page)
        next_offset = payload.get("next_offset")
        if next_offset is not None and next_offset != len(rows):
            raise ValueError(f"{path} advertises a next offset inconsistent with its rows")
        if "count" in payload and (
            not isinstance(payload["count"], int)
            or payload["count"] < len(rows)
            or (next_offset is None and payload["count"] != len(rows))
        ):
            raise ValueError(f"{path} does not match its declared catalogue size")
    if next_offset is not None:
        raise ValueError(f"{key} is missing pages starting at offset {next_offset}")
    return {key: rows}


def full_contracts(payload, key, target):
    contracts = {}
    identities = set()
    spans = set()
    for row in payload[key]:
        if not isinstance(row, dict):
            raise ValueError(f"{key} requires detail=full for contract verification")
        identifier = row["id"]
        contract = row["ontological_contract"]
        stable = contract["stable_id"]
        identity = (stable["sub_namespace_id"], stable["func_id"])
        if identity in identities:
            raise ValueError(f"{key} repeats the stable ABI identity for {identifier}")
        identities.add(identity)
        for name in ("precondition", "postcondition", "invariant"):
            fact = contract["nquin48_triples"][name]
            start, extent = fact["span"]
            if fact["parity_verified"] is not True or extent != 1 or start in spans:
                raise ValueError(f"Invalid or repeated {name} contract fact for {identifier}")
            spans.add(start)
        for bounds in contract["param_qudt_bounds"]:
            if not bounds["qudt_unit_iri"].startswith("http://qudt.org/vocab/unit/") or (
                bounds["qudt_dimension_iri"] is None
                or bounds["qudt_conversion_multiplier"] is None
            ):
                raise ValueError(f"Unresolved QUDT unit for {identifier}.{bounds['name']}")
        contracts[identifier] = contract
    if spans != set(range(3 * target)):
        raise ValueError(f"{key} does not cover the {3 * target} required NQuin indices")
    return contracts


def exposure_counts(payload):
    modules = payload.get("modules")
    if not isinstance(modules, list) or not modules or not payload.get("total_files_scanned"):
        raise ValueError("Exposure report is missing a nonempty module inventory")
    counts = Counter(module["bucket"] for module in modules)
    if "bucket_counts" not in payload:
        raise ValueError("Exposure report is missing its bucket counts")
    claimed_counts = Counter(
        {bucket: count for bucket, count in payload["bucket_counts"].items() if count}
    )
    if counts != claimed_counts:
        raise ValueError("Exposure report bucket totals disagree with its module inventory")
    citation_only = [
        module for module in modules
        if module["bucket"] == "CoveredOrCatalogued" and module["synth_reached_fns"] == 0
    ]
    return counts, citation_only


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--operations", nargs="+", type=Path, required=True,
                        help="Saved synth_operations JSON pages in offset order")
    parser.add_argument("--ontology", nargs="+", type=Path, required=True,
                        help="Saved synth_ontology_morphisms JSON pages in offset order")
    parser.add_argument("--exposure", type=Path, required=True, help="rust-audit exposure-gaps.json")
    parser.add_argument("--target-count", type=int, choices=(390, 400), default=390,
                        help="The milestone count. Defaults to Gate A.1")
    parser.add_argument("--strict-contracts", action="store_true",
                        help="Compare full descriptors, ABI IDs, NQuin spans and declared QUDT sidecars")
    args = parser.parse_args()
    try:
        operation_pages = join_catalogue_pages(args.operations, "operations")
        ontology_pages = join_catalogue_pages(args.ontology, "morphisms")
        operations = catalogue_ids(operation_pages, "operations")
        ontology = catalogue_ids(ontology_pages, "morphisms")
        if args.strict_contracts:
            operation_contracts = full_contracts(operation_pages, "operations", args.target_count)
            ontology_contracts = full_contracts(ontology_pages, "morphisms", args.target_count)
            if operation_contracts != ontology_contracts:
                raise ValueError("Operation and ontology ABI contract descriptors differ")
        counts, cited_only = exposure_counts(read_payload(args.exposure))
    except (KeyError, TypeError, ValueError, OSError) as error:
        parser.exit(1, f"WP-25 verifier: {error}\n")

    failures = []
    if len(operations) != args.target_count or len(ontology) != args.target_count:
        failures.append(
            f"Expected {args.target_count} IDs in each catalogue, got {len(operations)} operations "
            f"and {len(ontology)} morphisms"
        )
    if operations != ontology:
        operations_only = ", ".join(sorted(operations - ontology)) or "none"
        ontology_only = ", ".join(sorted(ontology - operations)) or "none"
        failures.append(f"Operation IDs absent from ontology: {operations_only}")
        failures.append(f"Ontology IDs absent from operations: {ontology_only}")
    if counts[DARK_BUCKET]:
        failures.append(f"{counts[DARK_BUCKET]} modules remain completely dark and uncatalogued")
    if failures:
        parser.exit(1, "\n".join(failures) + "\n")

    print(f"WP-25 catalogue and exposure checks passed for {args.target_count} matching IDs")
    print(f"Review actual AssetHost dispatch for {len(cited_only)} modules with no scanner-reached Synth functions")


if __name__ == "__main__":
    main()
