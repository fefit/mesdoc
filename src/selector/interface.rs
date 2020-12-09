pub trait NodeTrait {
	type NodeList: NodeListTrait;
	fn parent(&self) -> Result<Self::NodeList, &'static str>;
}

pub trait NodeListTrait: IntoIterator<Item = <Self as NodeListTrait>::Node> {
	type Node: NodeTrait;
	fn length(&self) -> usize;
	fn item(&self, index: usize) -> Option<Self::Node>;
}
