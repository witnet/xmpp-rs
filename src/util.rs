use minidom::Element;

pub trait FromElement where Self: Sized {
    type Err;

    fn from_element(elem: &Element) -> Result<Self, Self::Err>;
}

pub trait FromParentElement where Self: Sized {
    type Err;

    fn from_parent_element(elem: &Element) -> Result<Self, Self::Err>;
}

pub trait ToElement where Self: Sized {
    type Err;

    fn to_element(&self) -> Result<Element, Self::Err>;
}
