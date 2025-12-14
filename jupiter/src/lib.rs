//! Name*spacing* library.
//!
//! This is *not* meant to be the fastest most optimized library ever. It's meant to work well with
//! Draca, and other tools, I give no guarantees.
//!
//! [`Namespace`] is expected to have [`std::str`]s, and while this library is generic, I
//! specifically designed it to work with them, so I have no idea how other types work.
//!
//! # Key Terms
//!
//! ## Fragments
//! A fragment is `foo::**here**::bar`, `foo::here::**bar**`, `**foo**::here::bar`, etc. It is any
//! component in the namespace path.
//!
//! ## Value Fragment
//! A value fragment specifically a fragment that has a value in it.

use std::{
    borrow::Borrow, collections::BTreeMap, fmt::Write as _, marker::PhantomData, ops::Deref,
};

/// A global namespace holder.
pub struct Namespace<'a, S, T> {
    root: NamespaceNode<'a, S, T>,
    split: &'a str,
    // scopes: Vec<Frags<T>>,
}

#[derive(PartialEq)]
enum Root<S> {
    Root,
    Entry(S),
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub enum PathRules {
    SepPreceedsRoot,
    #[default]
    Default,
}

/// A fragment in the namespace.
pub struct NamespaceNode<'a, S, T> {
    name: Root<S>,
    value: Option<T>,
    split: &'a str,
    children: BTreeMap<S, Self>,
}

pub struct NodeValue<'a, S, T> {
    name: &'a S,
    value: &'a T,
}

trait FragIsRootTrait {}
trait FragIsRelativeTrait {}

#[doc(hidden)]
pub struct FragIsRoot;

#[doc(hidden)]
pub struct FragIsRelative;

impl FragIsRootTrait for FragIsRoot {}
impl FragIsRelativeTrait for FragIsRelative {}

#[derive(Debug)]
pub struct NamespaceFrags<'a, S, T> {
    frags: Vec<&'a S>,
    split: &'a str,
    _boo: PhantomData<T>,
}

impl<S, T> NodeValue<'_, S, T> {
    pub fn name(&self) -> &S {
        self.name
    }

    pub fn value(&self) -> &T {
        self.value
    }
}

impl<'a, S, T> Deref for NamespaceFrags<'a, S, T> {
    type Target = Vec<&'a S>;

    fn deref(&self) -> &Self::Target {
        &self.frags
    }
}

impl<'a, S, T> NamespaceFrags<'a, S, T> {
    fn from_vec_split(frags: Vec<&'a S>, split: &'a str) -> Self {
        Self {
            frags,
            split,
            _boo: PhantomData,
        }
    }
}

impl<S, T> NamespaceFrags<'_, S, T>
where
    S: ToString,
{
    fn string_doer(&self) -> String {
        let mut str = String::new();

        let mut iter = self.frags.iter().peekable();

        while let Some(next) = iter.next() {
            if iter.peek().is_none() {
                str += &next.to_string();
            } else {
                let _ = write!(str, "{}{}", next.to_string(), self.split);
            }
        }

        str
    }
}

#[allow(private_bounds)]
impl<S, T> NamespaceFrags<'_, S, T>
where
    S: ToString,
    T: FragIsRootTrait,
{
    pub fn as_absolute_path(&self, opts: PathRules) -> String {
        let mut str = self.string_doer();

        if matches!(opts, PathRules::SepPreceedsRoot) {
            str = format!("{}{str}", self.split);
        }

        str
    }
}

#[allow(private_bounds)]
impl<S, T> NamespaceFrags<'_, S, T>
where
    S: ToString,
    T: FragIsRelativeTrait,
{
    pub fn as_relative_path(&self) -> String {
        self.string_doer()
    }
}

impl<'a, S, T> Namespace<'a, S, T> {
    /// Create a new namespace with `S` namespace separator and `T` item.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new(split: &'a str) -> Self {
        Self {
            root: NamespaceNode::root(split),
            split,
        }
    }

    pub fn root(&self) -> &NamespaceNode<'_, S, T> {
        &self.root
    }

    pub const fn split(&self) -> &str {
        self.split
    }

    /// Get all value fragments.
    pub fn all_items(&'a self) -> Vec<NodeValue<'a, S, T>> {
        let mut out = vec![];
        self.root.collect_items(&mut out);
        out
    }
}

impl<S, T> Namespace<'_, S, T>
where
    S: PartialEq,
{
    /// Find all fragments where `item` is in them.
    ///
    /// This can return both fragments and value fragments.
    pub fn find<Q>(&self, item: Q) -> Vec<&NamespaceNode<'_, S, T>>
    where
        Q: Borrow<S>,
    {
        self.root.find(item.borrow())
    }
}

impl<S, T> Namespace<'_, S, T>
where
    S: Ord,
{
    /// Get `T` from a fragment iterator.
    pub fn get_item<I>(&self, iter: I) -> Option<&T>
    where
        I: IntoIterator<Item = S>,
    {
        self.root.get_item(iter)
    }

    pub fn get_namespace<I>(&self, iter: I) -> Option<&NamespaceNode<'_, S, T>>
    where
        I: IntoIterator<Item = S>,
    {
        self.root.wind_to_fragment(iter)
    }
}

impl<S, T> Namespace<'_, S, T>
where
    S: Ord + Clone,
{
    /// Insert a `T` at the location specified by `I`.
    ///
    /// # Note
    /// This will not automatically map a `T` to the end of `I`, such that:
    ///
    /// ```no_run
    /// ns.insert_at_module(["std", "fns"], hello); // ❌
    /// ```
    ///
    /// Will **not** produce `["std", "fns", "hello"] -> hello`, but instead `["std", "fns"] ->
    /// hello`
    ///
    /// Use [`Self::insert_at_module`] if you want that functionality.
    pub fn insert_at_module<I>(&mut self, iter: I, value: T)
    where
        I: IntoIterator<Item = S>,
    {
        let module = self.root.new_module(iter);
        module.emplace_item(value);
    }

    pub fn insert_in_module<I>(&mut self, iter: I, name: S, value: T)
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

impl<'a, S, T> NamespaceNode<'a, S, T> {
    const fn root(split: &'a str) -> Self {
        Self {
            name: Root::Root,
            value: None,
            split,
            children: BTreeMap::new(),
        }
    }

    fn leaf<I: Into<S>>(name: I, split: &'a str) -> Self {
        Self {
            name: Root::Entry(name.into()),
            value: None,
            split,
            children: BTreeMap::new(),
        }
    }

    /// Replace current node with a value fragment.
    fn emplace_item(&mut self, item: T) {
        self.value = Some(item);
        self.children = BTreeMap::new();
    }

    pub const fn name(&self) -> Option<&S> {
        match self.name {
            Root::Root => None,
            Root::Entry(ref e) => Some(e),
        }
    }

    /// Get the value of the current node.
    pub fn extract_value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn path_from_root(
        &'a self,
        root: &'a Namespace<S, T>,
    ) -> Option<NamespaceFrags<'a, S, FragIsRoot>> {
        self.path_inner(&root.root)
            .map(|f| NamespaceFrags::from_vec_split(f, root.split))
    }

    pub fn path_from_branch(
        &'a self,
        root: &'a Self,
    ) -> Option<NamespaceFrags<'a, S, FragIsRelative>> {
        self.path_inner(root)
            .map(|f| NamespaceFrags::from_vec_split(f, root.split))
    }

    fn path_inner(&'a self, root: &'a Self) -> Option<Vec<&'a S>> {
        let mut ret = vec![];
        if root.find_path_to(self, &mut ret) {
            Some(ret)
        } else {
            None
        }
    }

    fn find_path_to(&'a self, target: &'a Self, out: &mut Vec<&'a S>) -> bool {
        if std::ptr::eq(self, target) {
            return true;
        }

        for (name, child) in &self.children {
            if child.find_path_to(target, out) {
                out.insert(0, name);
                return true;
            }
        }

        false
    }

    fn as_value(&'a self) -> Option<NodeValue<'a, S, T>> {
        Some(NodeValue {
            name: match &self.name {
                Root::Entry(entr) => entr,
                Root::Root => return None,
            },
            value: match &self.value {
                Some(value) => value,
                None => return None,
            },
        })
    }

    fn collect_items(&'a self, out: &mut Vec<NodeValue<'a, S, T>>) {
        if self.value.is_some() {
            out.push(self.as_value().expect("value is some but name isn't?"));
        }

        for child in self.children.values() {
            child.collect_items(out);
        }
    }
}

impl<S, T> NamespaceNode<'_, S, T>
where
    S: PartialEq,
{
    /// Get all fragments matching `item`.
    fn find(&self, item: &S) -> Vec<&Self> {
        let mut out = vec![];
        self.find_inner(item, &mut out);
        out
    }

    fn find_inner<'a>(&'a self, item: &S, out: &mut Vec<&'a Self>) {
        if let Root::Entry(entr) = &self.name
            && *entr == *item
        {
            out.push(self);
        }

        for child in self.children.values() {
            child.find_inner(item, out);
        }
    }
}

impl<S, T> NamespaceNode<'_, S, T>
where
    S: Ord,
{
    fn get_item<I>(&self, iter: I) -> Option<&T>
    where
        I: IntoIterator<Item = S>,
    {
        self.wind_to_fragment(iter).and_then(|s| match s.name {
            Root::Entry(_) => s.extract_value(),
            Root::Root => None,
        })
    }

    /// Get the fragment at the module path.
    fn wind_to_fragment<I>(&self, iter: I) -> Option<&Self>
    where
        I: IntoIterator<Item = S>,
    {
        let mut cur = self;

        for part in iter {
            match cur.children.get(&part) {
                Some(next) => cur = next,
                None => return None,
            }
        }

        Some(cur)
    }
}

impl<S, T> NamespaceNode<'_, S, T>
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
                .or_insert_with(|| Self::leaf(module, node.split));
        }

        node
    }
}
