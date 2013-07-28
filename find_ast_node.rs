use syntax::ast;
use syntax::ast::*;
use syntax::ast_map;
use syntax::visit;
use syntax::visit::*;
use syntax::visit::{Visitor, fn_kind};
use syntax::codemap::*;
use rustc::{front, metadata, driver, middle};
use syntax::codemap::span;
use syntax::*;
use syntax::abi::AbiSet;
use rustc::middle::*;


// TODO: code here only depends on ty::ctxt, sess:Session is held in there aswell.
pub struct DocContext {
    crate: @ast::crate,
    tycx: middle::ty::ctxt,
    sess: driver::session::Session
}

macro_rules! dump{ ($($a:expr),*)=>
	(	{	let mut txt=file!()+":"+line!().to_str()+": " ; 
			$( txt=txt.append(
				fmt!("%s=%?",stringify!($a),$a)+",") 
			);*; 
			println(txt); 
		}
	)
}

/// main
pub fn find(c:@crate,_location:uint)->~[AstNode] {
	let mut s= @mut FindAstNodeSt{
		result:~[astnode_root], location:_location, stop:false
	};
	let vt=mk_vt(@Visitor{
		//visit_mod
		visit_view_item:f_view_item,
		//visit_foreign_item
		visit_item:f_item,
		visit_local:f_local,
		visit_block:f_block,
		visit_stmt:f_stmt,
		visit_arm:f_arm,
		visit_pat:f_pat,
		visit_decl:f_decl,
		visit_expr:f_expr,
		//visit_expr_post:f_expr,--called after visit
		visit_ty:f_ty,
		//visit_method
		//visit_trait_method
//		visit_struct_def:f_struct_def,
		visit_struct_field:f_struct_field,

		.. *default_visitor::<@mut FindAstNodeSt>()
		}
	);
	visit_crate(c, (s,vt));
	s.result.clone()
}

// TODO - is there an official wrapper like this for all nodes in libsyntax::ast?
// is there a way of acheiving this without one?
#[deriving(Clone)]
pub enum AstNode 
{
	astnode_mod(@_mod),
	astnode_view_item(@view_item),
	astnode_item(@item),
	astnode_local(@local),
	astnode_block(@blk),
	astnode_stmt(@stmt),
	astnode_arm(@arm),
	astnode_pat(@pat),
	astnode_decl(@decl),
	astnode_expr(@expr),
	astnode_expr_post(@expr),
	astnode_ty(@Ty),
	astnode_ty_method(@ty_method),
	astnode_trait_method(@trait_method),
	astnode_struct_def(@struct_def),
	astnode_struct_field(@struct_field),
	astnode_root	
}

pub trait AstNodeAccessors {
	pub fn get_id(&self)->Option<node_id>;
}

impl AstNodeAccessors for ast::decl_ {
	pub fn get_id(&self)->Option<node_id> {
		match *self{
			decl_local(ref x)=>Some(x.node.id),
			decl_item(ref x)=>Some(x.id)
		}
	}
}
impl<T:AstNodeAccessors> AstNodeAccessors for codemap::spanned<T> {
	pub fn get_id(&self)->Option<node_id> {
		self.node.get_id()
	}
}
impl AstNodeAccessors for ty_method {
	pub fn get_id(&self)->Option<node_id> {
		Some(self.id)
	}
}
impl AstNodeAccessors for view_item_ {
	pub fn get_id(&self)->Option<node_id> {
		match *self {
			view_item_extern_mod(_,_,node_id)=>Some(node_id),
			view_item_use(_)=>None
		}
	}
}

impl AstNodeAccessors for trait_method {
	pub fn get_id(&self)->Option<node_id> {
		match(*self) {
			required(ref m)=>Some(m.id),
			provided(ref m)=>None
		}
	}
}

impl AstNodeAccessors for AstNode {
	pub fn get_id(&self)->Option<node_id> {
		// todo - should be option<node_id> really..
		match *self {
			astnode_mod(ref x) => None,
			astnode_view_item(ref x) =>x.node.get_id(),
			astnode_item(ref x) =>Some(x.id),
			astnode_local(ref x) =>Some(x.node.id),
			astnode_block(ref x)=>None,
			astnode_stmt(ref x)=>None,
			astnode_arm(ref x)=>None,
			astnode_pat(ref x)=>Some(x.id),
			astnode_decl(ref x)=>x.get_id(),
			astnode_expr(ref x)=>Some(x.id),
			astnode_expr_post(ref x)=>Some(x.id),
			astnode_ty(ref x)=>Some(x.id),
			astnode_ty_method(ref x)=>Some(x.id),
			astnode_trait_method(ref x)=>x.get_id(),
			astnode_struct_def(ref x)=>None,
			astnode_struct_field(ref x)=>Some(x.node.id),
			astnode_root=>None,
		}
	}
}

struct FindAstNodeSt {
	result: ~[AstNode],		// todo - full tree path, all the parent nodes.
	location: uint,
	stop: bool
}
fn span_contains(l:uint,s:span)->bool {
	let BytePos(lo)=s.lo;
	let BytePos(hi)=s.hi;
	l>=lo && l<hi
}

// struct Visitor<E>.visit_mod: @fn(&_mod, span, node_id, (E, vt<E>)),
 //fn f_mod(a:&_mod, (s,v):(@mut State, vt<@mut State>)) {
	//if span_contains(s.location, a.span) {
	//	s.result.push(astnode_mod(@copy *a));
	//}
	//visit_struct_field(a,(s,v))
   //}
// struct Visitor<E>.visit_view_item: @fn(&view_item, (E, vt<E>)),
type FindAstNodeSV = (@mut FindAstNodeSt, vt<@mut FindAstNodeSt>);
fn f_view_item(a:&view_item, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_view_item(@ copy *a));
	}
	visit_view_item(a,(s,v))
}// struct Visitor<E>.visit_foreign_item: @fn(@foreign_item, (E, vt<E>)),
// struct Visitor<E>.visit_item: @fn(@item, (E, vt<E>)),
fn f_item(a:@item, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_item(a));
	}
	visit_item(a,(s,v))
}
// struct Visitor<E>.visit_local: @fn(@local, (E, vt<E>)),
fn f_local(a:@local, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_local(a));
	}
	visit_local(a,(s,v))
}

// struct Visitor<E>.visit_block: @fn(&blk, (E, vt<E>)),
fn f_block(a:&blk, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_block(@ copy *a));
	}
	visit_block(a,(s,v))
}

// struct Visitor<E>.visit_stmt: @fn(@stmt, (E, vt<E>)),
fn f_stmt(a:@stmt, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_stmt(a));
	}
	visit_stmt(a,(s,v))
}

// struct Visitor<E>.visit_arm: @fn(&arm, (E, vt<E>)),
fn f_arm(a:&arm, (s,v):FindAstNodeSV) {
// the whole arm doesn't have a span 
//	if span_contains(s.location, a.span) {
//		s.result.push(astnode_arm(@copy *a));
//	}
	visit_arm(a,(s,v))
}
// struct Visitor<E>.visit_pat: @fn(@pat, (E, vt<E>)),
fn f_pat(a:@pat, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_pat(a));
	}
	visit_pat(a,(s,v))
}


// struct Visitor<E>.visit_decl: @fn(@decl, (E, vt<E>)),
fn f_decl(a:@decl, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_decl(a));
	}
	visit_decl(a,(s,v))
}


// struct Visitor<E>.visit_expr: @fn(@expr, (E, vt<E>)),
fn f_expr(a:@expr, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_expr(a));
	}
	visit_expr(a,(s,v))
}

// struct Visitor<E>.visit_expr_post: @fn(@expr, (E, vt<E>)),
fn f_expr_post(a:@expr, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_expr_post(a));
	}
	visit_expr(a,(s,v))
}

// struct Visitor<E>.visit_ty: @fn(&Ty, (E, vt<E>)),
fn f_ty(a:&Ty, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_ty(@copy *a));
	}
	visit_ty(a,(s,v))
}

// struct Visitor<E>.visit_generics: @fn(&Generics, (E, vt<E>)),
// struct Visitor<E>.visit_fn: @fn(&fn_kind, &fn_decl, &blk, span, node_id, (E, vt<E>)),
fn f_fn(fk:&fn_kind, fd:&fn_decl, body:&blk, sp:span, nid:node_id, (s,v):FindAstNodeSV) {
	//if span_contains(s.location, span) {
		//s.result.push(astnode_fn2(a));
	//}
	visit_fn(fk,fd,body,sp,nid,(s,v))
}
// struct Visitor<E>.visit_ty_method: @fn(&ty_method, (E, vt<E>)),
// struct Visitor<E>.visit_trait_method: @fn(&trait_method, (E, vt<E>)),
// struct Visitor<E>.visit_struct_def: @fn(@struct_def, ident, &Generics, node_id, (E, vt<E>)),
 // fn f_struct_def(a:@struct_def, (s,v):(@mut State, vt<@mut State>)) {
 //	if span_contains(s.location, a.span) {
 //		s.result.push(astnode_struct_def(a));
 //	}
 //	visit_struct_field(a,(s,v))
 //}
// struct Visitor<E>.visit_struct_field: @fn(@struct_field, (E, vt<E>)),
fn f_struct_field(a:@struct_field, (s,v):FindAstNodeSV) {
	if span_contains(s.location, a.span) {
		s.result.push(astnode_struct_field(a));
	}
	visit_struct_field(a,(s,v))
}


pub fn get_node_info_str(dc:&DocContext,node:&[AstNode])->~str
{
	fn path_to_str(dc:&DocContext, path:&ast::Path)->~str
	{
		let mut acc=~"";
		let mut first=true;
		for path.idents.iter().advance |x|{
			if !first {acc=acc.append(~"::");}
			acc=acc.append(dc.sess.str_of(*x));
			first=false
		}
		acc
		// typeparams too... path.types?
	}
	fn pat_to_str(dc:&DocContext,p:&ast::pat)->~str{
		// todo -factor out and recurse
		match p.node {
			ast::pat_ident(bind_mode,ref path, opt)=>~"pat_ident:"+path_to_str(dc,path),
			ast::pat_enum(ref path,ref efields)=>~"pat_enum:"+path_to_str(dc,path),//	`todo-fields..
			ast::pat_struct(ref path,ref sfields,b)=>~"pat_struct:"+path_to_str(dc,path)+~"{"+sfields.map(|x|pat_to_str(dc,x.pat)+~",").to_str()+~"}",
			ast::pat_tup(ref elems)=>~"pat_tupl:"+elems.map(|&x|pat_to_str(dc,x)).to_str(),
			//ast::pat_box(ref box)=>~"box",
			ast::pat_uniq(ref u)=>~"uniq",
			ast::pat_region(ref p)=>~"rgn",
			ast::pat_lit(ref e)=>~"literal",
			ast::pat_range(ref e_start,ref e_end)=>~"range",
		
			_=>~"?"
		}
	};
	fn ty_to_str(dc:&DocContext,t:&ast::Ty)->~str{
		match t.node{
			ast::ty_nil=> ~"nil",
			ast::ty_bot=>~"bottomtype",
			ast::ty_box(ref mt)=>~"box",
			ast::ty_vec(ref mt)=>~"vec",
			ast::ty_fixed_length_vec(ref mt,ref expr)=>~"[T,..N]",
			ast::ty_ptr(ref mt)=>~"*",
			ast::ty_rptr(ref lifetime,ref mt)=>~"&",
			ast::ty_tup(ref types)=>~"("+types.map(|x|ty_to_str(dc,x)).to_str()+")", //todo: factor this out, map..
			ast::ty_path(ref path,ref params,node_id)=>~"path:id="+node_id.to_str()+" "+path_to_str(dc,path)
			,
		
			ast::ty_infer=>~"infered",
			_ =>~"?"
		}
	}
	fn expr_to_str(dc:&DocContext, x:&expr_)->~str {
		match *x {
			expr_struct(ref p,_,_)=>~"(expr_struct "+ path_to_str(dc,p) +")",
			expr_call(ref e,ref args,_)=>~"(expr_call("+expr_to_str(dc,&e.node)+args.map(|x|expr_to_str(dc,&x.node)).to_str()+~")",
			expr_field(ref e, ref i, ref tys)=>~"(expr_field("+expr_to_str(dc,&e.node)+")"+dc.sess.str_of(*i)+tys.map(|x|ty_to_str(dc,x)).to_str()+~")",
			_=>~"expr"
		}
	}


	match node.last() {
//			TODO -factor out repeatedly used functions here..
//			fn astnode_pat_to_str(&astnode_pat(x))->~str
//			fn path_to_str(&astnode_pat(x))->~str
//			fn expr_to_str(&astnode_pat(x))->~str

		&astnode_view_item(x)=>~"view_item: ?",
		&astnode_item(x)=>~"item: "+
			~"id="+x.id.to_str()+~" "+
			dc.sess.str_of(x.ident)+
			match x.node {
				ast::item_fn(ref fn_decl,_,_,_,_) =>~" fn_decl",
				ast::item_struct(ref struct_def,_) =>~" struct_def",
				_=>~"item_unknown"
			},

		&astnode_local(x)=>~"local: ?",
		&astnode_block(x)=>~"block: ?",
		&astnode_stmt(x)=>~"stmt: ?",
		&astnode_arm(x)=>~"arm: ?",
		&astnode_struct_field(sf)=>
			~"id="+sf.node.id.to_str()+~" "+
			match(sf.node.kind){
				ast::named_field(nf,vis)=>"struct named_field: "+dc.sess.str_of(nf)+" ",
				_=>~"struct anon_field"
			}+
			~":"+ty_to_str(dc,&sf.node.ty)/*sf.node.ty ..parse it.. */,
		&astnode_pat(p)=>~"pattern: "+
			~"id="+p.id.to_str()+~" "+
			pat_to_str(dc,p)
		,
		&astnode_decl(x)=>~"decl: ?",
		&astnode_ty(x)=>~"type: "+ty_to_str(dc,x),
		&astnode_struct_def(sf)=>~"struct def",
		&astnode_expr(x)=>expr_to_str(dc,&x.node),
		_=>	~"unknown"
	}
}

pub fn safe_node_id_to_type(cx: ty::ctxt, id: ast::node_id) -> Option<ty::t> {
    //io::println(fmt!("%?/%?", id, cx.node_types.len()));
    match cx.node_types.find(&(id as uint)) {
       Some(&t) => Some(t),
       None => None    
	}
}

