use std::mem;

pub struct List<T> {
    head: Link<T>,

}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None }
    }

    pub fn push(&mut self, elem: T) {
        let new_node: Box<Node<T>> = Box::new(Node {
            elem: elem,
            next: self.head.take(),
        });

        self.head = Some(new_node);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            node.elem
        })
    }

    fn pop_node(&mut self) -> Link<T> {
        self.head.take().map(|node| {
            self.head = node.next;
            Box::new(Node{elem: node.elem, next: None})
        })
    }

    fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| {
            &node.elem
        })
    }

    fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| {
            &mut node.elem
        })
    }

}

pub struct IntoIter<T>(List<T>);

impl<T> List<T> {
    fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        // access fields of a tuple struct numerically
        self.0.pop()
    }
}

// Iter is generic over *some* lifetime, it doesn't care
pub struct Iter<'a, T> {
    next: Option<&'a Node<T>>,
}

// No lifetime here. List doesn't have any associated lifetimes
impl<T> List<T> {
    // We declare a fresh lifetime here fore the _exact_ borrow that
    //  creates the iter. Now &self needs to be valid as long as the
    //  Iter is around.
    fn iter<'a>(&'a self) -> Iter<'a, T> {
        // `as_deref` is essentially `.map(|node| &**node)`
        // Rust normally does _deref coercion_ where it inserts
        // those *'s throughout your code to make it type check
        // It can do that because the borrow checker ensures we
        // never mess up pointers
        // In this case though, because the clojure is in
        // conjunction with the fact that we have `Option<&T>`
        // instead of `&T`, it's too complicated for it to
        // figure out, so we need to be explicit
        Iter { next: self.head.as_deref() }
    }
}

// We _do_ have a lifetime here, because Iter has one that we need
// to define
impl<'a, T> Iterator for Iter<'a, T> {
    // Need it here too, this is a type declaration
    type Item = &'a T;

    // This doesn't need to be changed. Handled by the above
    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|node| {
            // This works
            // self.next = node.next.as_deref();

            // But so does this! The turbofis ::<> tells the compiler
            // what we thing the types of the generics should be.
            // In this case `::<&Node<T>, _>` says it should return a
            // `&Node<T>`, and no other type
            // This lets the compiler know that `&node` should have a
            // deref coercion applied to it, so we don't need to
            // manually apply **
            self.next = node.next.as_ref().map::<&Node<T>, _>(|node| &node);

            &node.elem
        })
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut cur_link = self.head.take();
        // Do this until pattern doesn't match
        while let Some(mut boxed_node) = cur_link {
            cur_link = mem::replace(&mut boxed_node.next, None);
            // boxed_node goes out of scope and gets dropped here
            // but it's Node's `next` field that has been set to Link::Empty
            // so no unbounded recursion occurrs.
        }
    }
}

pub struct IterMut<'a, T> {
    next: Option<&'a mut Node<T>>,
}

impl<T> List<T> {
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut { next: self.head.as_deref_mut() }
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // `take` will give us exclusive access to the mutable ref
        self.next.take().map(|node| {
            self.next = node.next.as_deref_mut();
            &mut node.elem
        })
    }
}

#[cfg(test)]
mod tests {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        assert_eq!(list.pop(), None);

        list.push(1);
        list.push(2);
        list.push(3);

        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(2));

        list.push(4);
        list.push(5);

        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), Some(4));
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert_eq!(list.peek(), None);
        assert_eq!(list.peek_mut(), None);

        list.push(1); list.push(2); list.push(3);

        assert_eq!(list.peek(), Some(&3));
        assert_eq!(list.peek_mut(), Some(&mut 3));

        // this doesn't work. Writing the argument of the clojure this way
        //  doesn't specify that `value` is a mutable reference. Instead,
        //  it creates a pattern that will be matched against the argument.
        //  In other words, it means "the argument is a mutable reference,
        //  but just copy it into `value`.
        // list.peek_mut().map(|&mut value| {
        //     value = 42
        // });
        //  If we do `|value|`, it will be &mut i32, and we can mutate it!
        list.peek_mut().map(|value| {
            *value = 42
        });

        assert_eq!(list.peek(), Some(&42));
        assert_eq!(list.peek_mut(), Some(&mut 42));
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_mut() {
        let mut list = List::new();
        list.push(1); list.push(2); list.push(3);

        let mut iter = list.iter_mut();
        assert_eq!(iter.next(), Some(&mut 3));
        assert_eq!(iter.next(), Some(&mut 2));
        assert_eq!(iter.next(), Some(&mut 1));
        assert_eq!(iter.next(), None);
    }
}
