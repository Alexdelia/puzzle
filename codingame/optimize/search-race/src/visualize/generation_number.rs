use svg::node::element::Text;

const GENERATION_NUMBER_COLOR: &str = "white";

pub fn generation_number(generation: usize) -> Text {
	Text::new(format!("generation {generation}"))
		.set("x", 10)
		.set("y", 26)
		.set("font-family", "Arial")
		.set("font-size", 16)
		.set("opacity", 0.5)
		.set("fill", GENERATION_NUMBER_COLOR)
}
