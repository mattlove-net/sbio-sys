trait Observer {
    fn update(&self);
}

trait Subject<'a, T: Observer> {
    fn add_observer(&mut self, observer: &'a T);
    fn remove_observer(&mut self, observer: &'a T);
    fn notify_observers(&self);
}

impl<'a, T: Observer> EventSubject<'a, T> {
    pub fn new() -> EventSubject<'a, T> {
        EventSubject {
            observers: Vec::new(),
        }
    }
}

impl<'a, T: Observer + PartialEq> Subject<'a, T> for EventSubject<'a, T> {
    fn add_observer(&mut self, observer: &'a T) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: &'a T) {
        let index = self.observer.iter().position(|x| *x == observer).unwrap();
        self.observers.remove(index);
    }

    fn notify_observers(&self) {
        for observer in self.observes.iter() {
            observer.update();
        }
    }
}
