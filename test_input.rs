use std::*;

mod fruit {
	pub struct SomeStruct{
		red_value:int,green_value:int,blue_value:int
	}
}

fn foo(mut apples:fruit::SomeStruct,(oranges,lemon):(int,int))->int{
	let some_var_name=2*oranges;
	some_var_name-apples.red_value+lemon
}

fn main() {	
	use fruit::*;
	io::println(foo(SomeStruct{red_value:1,green_value:2,blue_value:3},(4,5)).to_str());
}
