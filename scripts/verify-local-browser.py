"""Verify an actual local Chromium page, retaining its result and screenshot."""
import argparse, base64, functools, http.server, json, subprocess, threading, time
from contextlib import ExitStack
from pathlib import Path
from urllib.parse import quote
from websockets.sync.client import connect

p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--root',type=Path,required=True)
p.add_argument('--out',type=Path,required=True)
p.add_argument('--browser',type=Path,default=Path('C:/Program Files/Google/Chrome/Application/chrome.exe'))
p.add_argument('--page',default='index.html')
p.add_argument('--timeout',type=int,default=90)
a=p.parse_args()
root=a.root.resolve(strict=True); out=a.out.resolve()
if not root.is_dir() or not a.browser.is_file(): p.error('Existing page root and browser required')
if not 1<=a.timeout<=180: p.error('Timeout must be 1..180 seconds')
page=(root/a.page).resolve(strict=True)
if not page.is_relative_to(root) or not page.is_file(): p.error('Page must be a file below the serving root')
out.mkdir(parents=True,exist_ok=False)
class Handler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Opener-Policy','same-origin')
        self.send_header('Cross-Origin-Embedder-Policy','require-corp')
        self.send_header('Cross-Origin-Resource-Policy','same-origin')
        super().end_headers()
    def log_message(self,*args): pass
server=http.server.ThreadingHTTPServer(('127.0.0.1',0),functools.partial(Handler,directory=str(root)))
threading.Thread(target=server.serve_forever,daemon=True).start()
profile=out/'browser-profile'
command=[str(a.browser),'--headless=new','--no-first-run','--no-default-browser-check','--disable-background-networking','--disable-sync','--disable-crash-reporter','--disable-breakpad','--remote-debugging-port=0','--user-data-dir='+str(profile),'about:blank']
report={'status':'running','browser':str(a.browser),'root':str(root),'page':a.page,'provider_calls':False}
process=None; sock=None; connections=ExitStack()
try:
    with (out/'browser-stderr.log').open('wb') as err:
        process=subprocess.Popen(command,stdout=subprocess.DEVNULL,stderr=err,creationflags=subprocess.CREATE_NO_WINDOW)
        deadline=time.monotonic()+a.timeout
        active=profile/'DevToolsActivePort'
        while not active.is_file():
            if process.poll() is not None: raise RuntimeError('Browser exited before local test connection')
            if time.monotonic()>=deadline: raise TimeoutError('Browser startup')
            time.sleep(.1)
        port,endpoint=active.read_text().splitlines()[:2]
        sock=connections.enter_context(connect('ws://127.0.0.1:'+port+endpoint,open_timeout=10,max_size=32*1024*1024))
        request_id=0
        def rpc(method,params=None,session=None):
            global request_id
            request_id+=1
            message={'id':request_id,'method':method,'params':params or {}}
            if session: message['sessionId']=session
            sock.send(json.dumps(message))
            while True:
                remaining=deadline-time.monotonic()
                if remaining<=0: raise TimeoutError(method)
                reply=json.loads(sock.recv(timeout=remaining))
                if reply.get('id')!=request_id: continue
                if 'error' in reply: raise RuntimeError(method+': '+json.dumps(reply['error']))
                return reply.get('result',{})
        report['browser_version']=rpc('Browser.getVersion')
        target=rpc('Target.createTarget',{'url':'about:blank'})['targetId']
        session=rpc('Target.attachToTarget',{'targetId':target,'flatten':True})['sessionId']
        rpc('Page.enable',session=session)
        rpc('Emulation.setDeviceMetricsOverride',{'width':1280,'height':1600,'deviceScaleFactor':1,'mobile':False},session)
        url='http://127.0.0.1:'+str(server.server_port)+'/'+quote(page.relative_to(root).as_posix())
        rpc('Page.navigate',{'url':url},session)
        while True:
            value=rpc('Runtime.evaluate',{'expression':'JSON.stringify(window.emfReview || null)','returnByValue':True},session)
            raw=value.get('result',{}).get('value')
            result=json.loads(raw) if raw else None
            if result and result.get('status')!='running': break
            if time.monotonic()>=deadline: raise TimeoutError('Page review did not finish')
            time.sleep(.2)
        report['page_result']=result
        screenshot=rpc('Page.captureScreenshot',{'format':'png','captureBeyondViewport':True},session)
        (out/'browser.png').write_bytes(base64.b64decode(screenshot['data'],validate=True))
        html=rpc('Runtime.evaluate',{'expression':'document.documentElement.outerHTML','returnByValue':True},session)
        (out/'page.html').write_text(html['result']['value'],encoding='utf-8')
        report['status']='passed' if result.get('status')=='passed' else 'failed'
except Exception as error:
    report['status']='failed'; report['error']=str(error)
finally:
    if sock:
        try: sock.send(json.dumps({'id':999999,'method':'Browser.close'}))
        except Exception: pass
    connections.close()
    if process:
        try: process.wait(timeout=10)
        except subprocess.TimeoutExpired:
            # Only the process tree launched above, never an existing user browser.
            subprocess.run(['taskkill','/PID',str(process.pid),'/T','/F'],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW)
            process.wait(timeout=10)
    server.shutdown(); server.server_close()
    (out/'results.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(out)
raise SystemExit(0 if report['status']=='passed' else 1)
