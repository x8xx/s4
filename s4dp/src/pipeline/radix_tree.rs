pub struct RadixTree<'a> {
    root: &'a Node<'a>,
}

struct Node<'a> {
    left: Option<&'a Node<'a>>,
    right: Option<&'a Node<'a>>,
}
