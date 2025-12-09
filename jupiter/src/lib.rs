//! Name*spacing* library.

use std::collections::BTreeMap;

pub trait NamespaceSeparator {
    fn sep(&self) -> &str;
}

pub struct Namespace<S, T> {
    root: NamespaceNode<S, T>,
    // scopes: Vec<Frags<T>>,
}

enum Root<S> {
    Root,
    Entry(S),
}

struct NamespaceNode<S, T> {
    name: Root<S>,
    value: Option<T>,
    children: BTreeMap<S, Self>,
}

impl<S, T> Namespace<S, T> {
    /// Create a new namespace with `S` namespace separator and `T` item.
    pub fn new() -> Self {
        Self {
            root: NamespaceNode::root(),
        }
    }
}

impl<S, T> Namespace<S, T>
where
    S: Ord + Clone,
{
    /// Insert a `T` at the location specified by `I`.
    ///
    /// # Note
    /// This will not automatically map a `T` to the end of `I`, such that:
    ///
    /// ```rust
    /// ns.insert_at_module(["std", "fns"], hello);
    /// ```
    ///
    /// Will **not** produce `["std", "fns", "hello"] -> hello`, but instead `["std", "fns"] ->
    /// hello`
    ///
    /// Use [`Self::insert_with_name`] if you want that functionality.
    pub fn insert_at_module<I>(&mut self, iter: I, value: T)
    where
        I: IntoIterator<Item = S>,
    {
        let module = self.root.new_module(iter);
        module.emplace_item(value);
    }

    pub fn insert_with_name<I>(&mut self, iter: I, name: S, value: T)
    where
        I: IntoIterator<Item = S>,
    {
        let module = self.root.new_module(iter);
        module.insert_with_name(name, value);
    }

    pub fn new_module<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = S>,
    {
        self.root.new_module(iter);
    }
}

impl<S, T> NamespaceNode<S, T> {
    fn root() -> Self {
        Self {
            name: Root::Root,
            value: None,
            children: BTreeMap::new(),
        }
    }

    fn leaf<I: Into<S>>(name: I) -> Self {
        Self {
            name: Root::Entry(name.into()),
            value: None,
            children: BTreeMap::new(),
        }
    }

    fn emplace_item(&mut self, item: T) {
        self.value = Some(item);
        self.children = BTreeMap::new();
    }
}

impl<S, T> NamespaceNode<S, T>
where
    S: Ord + Clone,
{
    fn insert_with_name(&mut self, name: S, item: T) {
        let module = self.new_module([name]);
        module.emplace_item(item);
    }

    fn new_module<I>(&mut self, module_path: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
    {
        let mut node = self;

        for module in module_path {
            node = node
                .children
                .entry(module.clone())
                .or_insert_with(|| Self::leaf(module.clone()));
        }

        node
    }
}
