//! Name*spacing* library.
//!
//! # Key Terms
//!
//! ## Fragments
//! A fragment is `foo::**here**::bar`, `foo::here::**bar**`, `**foo**::here::bar`, etc. It is any
//! component in the namespace path.
//!
//! ## Value Fragment
//! A value fragment specifically a fragment that has a value in it.

use std::{collections::BTreeMap, marker::PhantomData, ops::Deref};

pub trait NamespaceSeparator {
    fn sep(&self) -> &str;
}

/// A global namespace holder.
pub struct Namespace<S, T> {
    root: NamespaceNode<S, T>,
    // scopes: Vec<Frags<T>>,
}

#[derive(PartialEq)]
enum Root<S> {
    Root,
    Entry(S),
}

#[derive(Clone, Copy)]
pub enum PathRules {
    SepPreceedsRoot,
    Default,
}

/// A fragment in the namespace.
pub struct NamespaceNode<S, T> {
    name: Root<S>,
    value: Option<T>,
    children: BTreeMap<S, Self>,
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
    _boo: PhantomData<T>,
}

impl<'a, S, T> Deref for NamespaceFrags<'a, S, T> {
    type Target = Vec<&'a S>;

    fn deref(&self) -> &Self::Target {
        &self.frags
    }
}

impl<'a, S, T> From<Vec<&'a S>> for NamespaceFrags<'a, S, T> {
    fn from(frags: Vec<&'a S>) -> Self {
        Self {
            frags,
            _boo: PhantomData,
        }
    }
}

impl<'a, S, T> NamespaceFrags<'a, S, T>
where
    S: ToString,
{
    fn string_doer(&self, sep: &str) -> String {
        let mut str = String::new();

        let mut iter = self.frags.iter().peekable();

        while let Some(next) = iter.next() {
            if iter.peek().is_none() {
                str += &next.to_string();
            } else {
                str += &format!("{}{}", next.to_string(), sep);
            }
        }

        str
    }
}

#[allow(private_bounds)]
impl<'a, S, T> NamespaceFrags<'a, S, T>
where
    S: ToString,
    T: FragIsRootTrait,
{
    pub fn as_absolute_path(&self, sep: &str, opts: PathRules) -> String {
        let mut str = self.string_doer(sep);

        if matches!(opts, PathRules::SepPreceedsRoot) {
            str = format!("{sep}{str}")
        }

        str
    }
}

#[allow(private_bounds)]
impl<'a, S, T> NamespaceFrags<'a, S, T>
where
    S: ToString,
    T: FragIsRelativeTrait,
{
    pub fn as_relative_path(&self, sep: &str) -> String {
        self.string_doer(sep)
    }
}

impl<S, T> Namespace<S, T> {
    /// Create a new namespace with `S` namespace separator and `T` item.
    #[must_use]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            root: NamespaceNode::root(),
        }
    }

    pub fn root(&self) -> &NamespaceNode<S, T> {
        &self.root
    }
}

impl<S, T> Namespace<S, T>
where
    S: PartialEq,
{
    /// Find all fragments where `item` is in them.
    pub fn find(&self, item: &S) -> Vec<&NamespaceNode<S, T>> {
        self.root.find(item)
    }
}

impl<S, T> Namespace<S, T>
where
    S: Ord + PartialEq,
{
    pub fn get_namespace<I>(&self, iter: I) -> Option<&NamespaceNode<S, T>>
    where
        I: IntoIterator<Item = S>,
    {
        self.root.wind_to_fragment(iter)
    }
}

impl<S, T> Namespace<S, T>
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
    /// ```no_run
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
    const fn root() -> Self {
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

    /// Replace current node with a value fragment.
    fn emplace_item(&mut self, item: T) {
        self.value = Some(item);
        self.children = BTreeMap::new();
    }

    /// Get the value of the current node.
    fn extract_value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn path_from_root<'a>(
        &'a self,
        root: &'a Namespace<S, T>,
    ) -> Option<NamespaceFrags<'a, S, FragIsRoot>> {
        self.path_inner(&root.root).map(Into::into)
    }

    pub fn path_from_branch<'a>(
        &'a self,
        root: &'a Self,
    ) -> Option<NamespaceFrags<'a, S, FragIsRelative>> {
        self.path_inner(root).map(Into::into)
    }

    fn path_inner<'a>(&'a self, root: &'a Self) -> Option<Vec<&'a S>> {
        let mut ret = vec![];
        if root.find_path_to(self, &mut ret) {
            Some(ret)
        } else {
            None
        }
    }

    fn find_path_to<'a>(&'a self, target: &'a Self, out: &mut Vec<&'a S>) -> bool {
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
}

impl<S, T> NamespaceNode<S, T>
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
        match &self.name {
            Root::Root => (),
            Root::Entry(entr) => {
                if *entr == *item {
                    out.push(self);
                }
            }
        }

        for child in self.children.values() {
            child.find_inner(item, out);
        }
    }
}

impl<S, T> NamespaceNode<S, T>
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
