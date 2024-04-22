use std::sync::{Arc, Mutex};

pub trait IObserver<T> {
    fn update(&mut self, event: &T);
}

pub type Observer<T> = Arc<Mutex<dyn IObserver<T>>>;

pub trait ISubject<T> {
    fn add_observer(&mut self, observer: Observer<T>);
    fn remove_observer(&mut self, observer: Observer<T>);
    fn notify_observers(&self, data: &T);
    fn len(&self) -> usize;
}

pub struct Subject<T> {
    observers: Vec<Observer<T>>,
}

impl<T> Subject<T> {
    pub fn new() -> Subject<T> {
        Subject {
            observers: Vec::new(),
        }
    }
}

impl<T> ISubject<T> for Subject<T> {
    fn add_observer(&mut self, observer: Observer<T>) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: Observer<T>) {
        let index = self
            .observers
            .iter()
            .position(|x| Arc::ptr_eq(x, &observer))
            .unwrap();
        self.observers.remove(index);
    }

    fn notify_observers(&self, event: &T) {
        //self.observers.iter_mut().map(|observer| {observer.update(data);});
        for observer_handle in self.observers.clone() {
            let mut observer = observer_handle
                .lock()
                .expect("Failed to acquire observer lock!");
            observer.update(event);
        }
    }

    fn len(&self) -> usize {
        self.observers.len()
    }
}

#[cfg(test)]
mod oberserver_tests {
    use super::*;

    struct TestEvent {
        data: u32,
    }

    struct TestObserver {
        data: u32,
    }

    impl IObserver<TestEvent> for TestObserver {
        fn update(&mut self, event: &TestEvent) {
            self.data = event.data;
        }
    }

    #[test]
    fn add_observer_test() {
        let mut subject = Subject::new();
        let observer_a = Arc::new(Mutex::new(TestObserver { data: 0 }));
        let observer_b = Arc::new(Mutex::new(TestObserver { data: 0 }));

        subject.add_observer(observer_a.clone());
        assert_eq!(subject.len(), 1);

        subject.add_observer(observer_b.clone());
        assert_eq!(subject.len(), 2);
    }

    #[test]
    fn remove_observer_test() {
        let mut subject = Subject::new();
        let observer_a = Arc::new(Mutex::new(TestObserver { data: 0 }));
        let observer_b = Arc::new(Mutex::new(TestObserver { data: 0 }));

        subject.add_observer(observer_a.clone());
        assert_eq!(subject.len(), 1);

        subject.add_observer(observer_b.clone());
        assert_eq!(subject.len(), 2);

        subject.remove_observer(observer_a);
        assert_eq!(subject.len(), 1);
    }

    #[test]
    fn notify_test() {
        let mut subject = Subject::new();
        let observer_a = Arc::new(Mutex::new(TestObserver { data: 0 }));
        let observer_b = Arc::new(Mutex::new(TestObserver { data: 0 }));

        subject.add_observer(observer_a.clone());
        assert_eq!(subject.len(), 1);

        subject.add_observer(observer_b.clone());
        assert_eq!(subject.len(), 2);

        let event1 = TestEvent { data: 10 };
        subject.notify_observers(&event1);
        assert_eq!(
            observer_a.lock().expect("Failed to lock mutex lock!").data,
            event1.data
        );
        assert_eq!(
            observer_b.lock().expect("Failed to lock mutex lock!").data,
            event1.data
        );
    }
}
