//! One owned cold operator lane; native credential-store work never runs in render.
use super::{
    runtime::Runtime,
    types::{LaunchAction, LaunchView},
};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::JoinHandle;
use std::time::Duration;
struct Request {
    action: LaunchAction,
    reply: futures_channel::oneshot::Sender<LaunchReply>,
}
pub struct LaunchReply {
    pub view: Option<LaunchView>,
    pub message: String,
}
#[derive(Clone)]
pub struct LaunchHandle {
    sender: mpsc::SyncSender<Request>,
    closing: Arc<AtomicBool>,
}
impl PartialEq for LaunchHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.closing, &other.closing)
    }
}
pub struct LaunchWorker {
    closing: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl LaunchWorker {
    pub fn start(mut runtime: Runtime) -> Result<(Self, LaunchHandle, LaunchView), String> {
        let initial = runtime.view()?;
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let closing = Arc::new(AtomicBool::new(false));
        let stop = closing.clone();
        let thread=std::thread::Builder::new().name("poet-native-project-operator".into()).spawn(move||{
            loop{
                if stop.load(Ordering::Acquire){break}
                let request=match receiver.recv_timeout(Duration::from_millis(50)){
                    Ok(r)=>r,Err(mpsc::RecvTimeoutError::Timeout)=>continue,Err(_)=>break,
                };
                if stop.load(Ordering::Acquire)||request.reply.is_canceled(){continue}
                let result=runtime.perform(request.action);
                let view=runtime.view();
                let reply=LaunchReply{message:match result{Ok(())=>"Native project operation completed. Review the current project/session state.".into(),Err(e)=>e},view:view.ok()};
                let _=request.reply.send(reply);
            }
            // Runtime's owned authoring worker is dropped/joined before this
            // operator thread exits; secrets and live session handles retire.
        }).map_err(|e|e.to_string())?;
        Ok((
            Self {
                closing: closing.clone(),
                thread: Some(thread),
            },
            LaunchHandle { sender, closing },
            initial,
        ))
    }
    pub fn finish(mut self) -> Result<(), String> {
        self.join()
    }
    fn join(&mut self) -> Result<(), String> {
        self.closing.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            thread.join().map_err(|_| {
                "Native project operator panicked; inspect retained state before retrying"
            })?;
        }
        Ok(())
    }
}
impl Drop for LaunchWorker {
    fn drop(&mut self) {
        if let Err(e) = self.join() {
            eprintln!("{e}")
        }
    }
}
pub async fn run(handle: LaunchHandle, action: LaunchAction) -> Result<LaunchReply, String> {
    super::types::validate_action(&action)?;
    if handle.closing.load(Ordering::Acquire) {
        return Err("Native project operator is closing".into());
    }
    let (reply, receive) = futures_channel::oneshot::channel();
    handle
        .sender
        .try_send(Request { action, reply })
        .map_err(|_| "Native project operator is busy or unavailable; operation not queued")?;
    receive.await.map_err(|_|"Native operator ended without a result; effects may have committed. Inspect state before retrying.".into())
}
