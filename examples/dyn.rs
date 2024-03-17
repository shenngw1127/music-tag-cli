trait Human {
    fn speak(&self);
}

#[derive(Debug)]
struct Person {
    name: String,
}

struct Account<'a> {
    person1: Box<dyn Human + 'a>,
    person2: Box<dyn Human + 'a>,
}

impl Human for Person {
    fn speak(&self) {
        println!("my name is {:?}", self.name);
    }
}

impl<'a> Account<'a> {
    fn x(&self) {
        self.person1.speak();
        self.person2.speak();
    }
}

fn main() {
    let person1 = Person {
        name: "Jing".to_owned(),
    };

    let person2 = Person {
        name: "2".to_owned(),
    };

    let account = Account {
        person1: Box::new(person1),
        person2: Box::new(person2),
    };

    account.x();
}
