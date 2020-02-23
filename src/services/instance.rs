use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};

pub struct ServiceInstance<O> {
    pub receiver: Option<Receiver<O>>,
    sender: Sender<O>,
}

impl<O> ServiceInstance<O> {
    pub fn new() -> ServiceInstance<O> {
        let (sender, receiver) = channel();
        ServiceInstance {
            receiver: Some(receiver),
            sender,
        }
    }

    pub fn sender(&self) -> Sender<O> {
        self.sender.clone()
    }

    pub fn receiver(&mut self) -> Receiver<O> {
        match self.receiver.take() {
            Some(receiver) => receiver,
            _ => {
                panic!("failed to extract receiver from service- is it already on?");
            }
        }
    }
}

// 1. Create the service instance struct (which creates a channel for you)
// 2. Run the service event loop method with a clone of the sender of all services it depends on
macro_rules! define_services {
    ($( (module: $service:path, name: $service_instance:ident, dependencies: [$($dependency:ident),*] $(, extras: [$($extra:ident),*])?)),*) => (
        $(let mut $service_instance = ServiceInstance::new();)*
        $(
            paste::expr! {
                $(let [<$dependency _clone>] = $dependency.sender();)*
                $($(let [<$extra _clone>] = $extra.clone();)*)?
                let sender = $service_instance.sender();
                let receiver = $service_instance.receiver();
                thread::spawn(move || $service(receiver, sender $(, {[<$dependency _clone>]})* $(, $({[<$extra _clone>]})*)? ));
            }
        )*
    );
}
