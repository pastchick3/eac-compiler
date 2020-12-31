use std::cell::RefCell;
use std::cmp::PartialEq;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use colored::*;
use indexmap::IndexMap;

/// Represent all errors that may occur during the transpilation.
#[derive(Debug, PartialEq)]
pub enum Error {
    Preprocessing { message: String, location: Location },
    Lexing { message: String, location: Location },
    Parsing { message: String, location: Location },
    Resolving { message: String, location: Location },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Preprocessing { message, location } => write!(
                f,
                "{} {}: {}",
                location,
                "Preprocessing Error".red(),
                message
            ),
            Error::Lexing { message, location } => {
                write!(f, "{} {}: {}", location, "Lexing Error".red(), message)
            }
            Error::Parsing { message, location } => {
                write!(f, "{} {}: {}", location, "Parsing Error".red(), message)
            }
            Error::Resolving { message, location } => {
                write!(f, "{} {}: {}", location, "Resolving Error".red(), message)
            }
        }
    }
}

/// Represent a specific location in the source file.
#[derive(Debug, Clone, Eq, Default)]
pub struct Location {
    pub file_name: String,
    pub line_no: usize,
    pub char_no: usize,
}

impl Location {
    pub fn new(file_name: &str, line_index: usize, char_index: usize) -> Location {
        // All `*_index` starts from 0, and all `*_no` starts from 1.
        Location {
            file_name: file_name.to_string(),
            line_no: line_index + 1,
            char_no: char_index + 1,
        }
    }
}

impl fmt::Display for Location {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}:{})", self.file_name, self.line_no, self.char_no)
    }
}

/// All `Location` will be compared equal, because they will never be directly
/// compared, but we want other structures which only different in `Location`
/// to be compared equal.
impl PartialEq for Location {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

/// Since we implement `PartialEq`, we cannot derive `Hash`.
/// Instead We (emptily) implement `Hash` to uphold the property
/// `k1 == k2 -> hash(k1) == hash(k2)`
impl Hash for Location {
    fn hash<H: Hasher>(&self, _: &mut H) {}
}

/// Traits for types that can locate itself in the source file.
pub trait Locate {
    fn locate(&self) -> Location;
}

/// Tokens used by the lexer and the parser.
#[derive(Debug, PartialEq)]
pub enum Token {
    Ident { literal: String, location: Location },
    IntConst { literal: String, location: Location },
    FloatConst { literal: String, location: Location },
    CharConst { literal: String, location: Location },
    StrConst { literal: String, location: Location },
    Comment { literal: String, location: Location },
    Include { literal: String, location: Location },

    Void(Location),
    Char(Location),
    Short(Location),
    Int(Location),
    Long(Location),
    Float(Location),
    Double(Location),
    Signed(Location),
    Unsigned(Location),

    Plus(Location),
    Minus(Location),
    Asterisk(Location),
    Slash(Location),
    Percent(Location),
    BiPlus(Location),
    BiMinus(Location),
    Equal(Location), // "="

    Small(Location),
    Large(Location),
    SmallEq(Location),
    LargeEq(Location),
    EqTo(Location), // "=="
    NotEqTo(Location),
    And(Location),
    Or(Location),
    Not(Location),

    PlusEq(Location),
    MinusEq(Location),
    AsteriskEq(Location),
    SlashEq(Location),
    PercentEq(Location),

    LParen(Location),
    RParen(Location),
    LBracket(Location),
    RBracket(Location),
    LBrace(Location),
    RBrace(Location),

    Switch(Location),
    Case(Location),
    Default_(Location),
    If(Location),
    Else(Location),
    Do(Location),
    While(Location),
    For(Location),
    Continue(Location),
    Break(Location),
    Return(Location),
    Struct(Location),

    Ampersand(Location),
    Dot(Location),
    Arrow(Location),
    Comma(Location),
    Colon(Location),
    Semicolon(Location),
    Ellipsis(Location),
}

impl Locate for Token {
    fn locate(&self) -> Location {
        use Token::*;

        match self {
            Ident { location, .. }
            | IntConst { location, .. }
            | FloatConst { location, .. }
            | CharConst { location, .. }
            | StrConst { location, .. }
            | Comment { location, .. }
            | Include { location, .. } => location.clone(),

            Void(loc) | Char(loc) | Short(loc) | Int(loc) | Long(loc) | Float(loc)
            | Double(loc) | Signed(loc) | Unsigned(loc) | Plus(loc) | Minus(loc)
            | Asterisk(loc) | Slash(loc) | Percent(loc) | BiPlus(loc) | BiMinus(loc)
            | Equal(loc) | Small(loc) | Large(loc) | SmallEq(loc) | LargeEq(loc) | EqTo(loc)
            | NotEqTo(loc) | And(loc) | Or(loc) | Not(loc) | PlusEq(loc) | MinusEq(loc)
            | AsteriskEq(loc) | SlashEq(loc) | PercentEq(loc) | LParen(loc) | RParen(loc)
            | LBracket(loc) | RBracket(loc) | LBrace(loc) | RBrace(loc) | Switch(loc)
            | Case(loc) | Default_(loc) | If(loc) | Else(loc) | Do(loc) | While(loc) | For(loc)
            | Continue(loc) | Break(loc) | Return(loc) | Struct(loc) | Ampersand(loc)
            | Dot(loc) | Arrow(loc) | Comma(loc) | Colon(loc) | Semicolon(loc) | Ellipsis(loc) => {
                loc.clone()
            }
        }
    }
}

/// The result of comparing two types.
///
/// Please refer to the README file for what these relationship mean.
#[derive(Debug)]
pub enum TypeRelationship {
    Sub,
    Equal,
    Super,
    Invalid,
}

/// AST nodes for types.
///
/// Please refer to the README file for the complete type hierarchy.
#[derive(Debug, Clone)]
pub enum Type {
    Any,
    T(Option<Box<Type>>), // whether it has been specialized to a concrete type
    Void(Option<Location>),
    Char(Option<Location>),
    Struct {
        name: String,
        members: IndexMap<String, Type>,
        location: Option<Location>,
    },
    Nothing,

    AnyRef,
    Pointer {
        refer: Box<Type>,
        location: Option<Location>,
    },
    Array {
        content: Box<Type>,
        length: Option<usize>,
        location: Option<Location>,
    },
    Null,

    Double(Option<Location>),
    Float(Option<Location>),
    Long(Option<Location>),
    UnsignedLong(Option<Location>),
    Int(Option<Location>),
    UnsignedInt(Option<Location>),
    Short(Option<Location>),
    UnsignedShort(Option<Location>),
    Byte,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        use Type::*;

        match (self, other) {
            (Any, Any) => true,
            (T(None), T(None)) => true,
            (T(Some(left)), T(Some(right))) => left == right,
            (Void(_), Void(_)) => true,
            (Char(_), Char(_)) => true,
            (
                Struct {
                    name: l_n,
                    members: l_mems,
                    ..
                },
                Struct {
                    name: r_n,
                    members: r_mems,
                    ..
                },
            ) => l_n == r_n && l_mems == r_mems,
            (Nothing, Nothing) => true,

            (AnyRef, AnyRef) => true,
            (Pointer { refer: left, .. }, Pointer { refer: right, .. }) => left == right,
            (Array { content: left, .. }, Array { content: right, .. }) => left == right,
            (Null, Null) => true,

            (Double(_), Double(_)) => true,
            (Float(_), Float(_)) => true,
            (Long(_), Long(_)) => true,
            (UnsignedLong(_), UnsignedLong(_)) => true,
            (Int(_), Int(_)) => true,
            (UnsignedInt(_), UnsignedInt(_)) => true,
            (Short(_), Short(_)) => true,
            (UnsignedShort(_), UnsignedShort(_)) => true,
            (Byte, Byte) => true,

            _ => false,
        }
    }
}

impl Locate for Type {
    fn locate(&self) -> Location {
        use Type::*;

        match self {
            Any | Nothing | AnyRef | Null | Byte => Location::default(),
            T(Some(type_)) => type_.locate(),
            T(None) => Location::default(),
            Void(location)
            | Char(location)
            | Struct { location, .. }
            | Pointer { location, .. }
            | Array { location, .. }
            | Double(location)
            | Float(location)
            | Long(location)
            | UnsignedLong(location)
            | Int(location)
            | UnsignedInt(location)
            | Short(location)
            | UnsignedShort(location) => location.clone().unwrap_or_default(),
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Type::*;

        match self {
            Any { .. } => write!(f, "Any"),
            T(Some(type_)) => write!(f, "T({})", type_),
            T(None) => write!(f, "T"),
            Void(_) => write!(f, "void"),
            Char(_) => write!(f, "char"),
            Struct { name, .. } => write!(f, "struct {}", name),
            Nothing { .. } => write!(f, "Nothing"),

            AnyRef { .. } => write!(f, "AnyRef"),
            Pointer { refer, .. } => write!(f, "*{}", refer),
            Array { content, .. } => write!(f, "{}[]", content),
            Null { .. } => write!(f, "Null"),

            Double(_) => write!(f, "double"),
            Float(_) => write!(f, "float"),
            Long(_) => write!(f, "long"),
            UnsignedLong(_) => write!(f, "unsigned long"),
            Int(_) => write!(f, "int"),
            UnsignedInt(_) => write!(f, "unsigned int"),
            Short(_) => write!(f, "short"),
            UnsignedShort(_) => write!(f, "unsigned short"),
            Byte => write!(f, "byte"),
        }
    }
}

impl Type {
    /// Specialize a `T` type to a concrete type. Return false
    /// if it is not a `T` type or it has been specialized.
    pub fn specialize(&mut self, type_: &Type) -> bool {
        match self {
            Type::T(specialized) => {
                specialized.replace(Box::new(type_.clone()));
                true
            }
            _ => false,
        }
    }

    /// Get the concrete type a `T` type. Return self if it is not `T`.
    pub fn specialized(&self) -> Option<Type> {
        match self {
            Type::T(Some(specialized)) => Some(*specialized.clone()),
            Type::T(None) => None,
            _ => Some(self.clone()),
        }
    }

    /// Specialize `lower` to a concrete type, based on its
    /// relationship with `upper`.
    pub fn specialize_dummy_type(upper: &Type, lower: &Type) -> Option<Type> {
        if !Type::is_dummy_type(lower) {
            return Some(lower.clone());
        }

        match lower {
            Type::Byte => Some(lower.clone()),
            Type::Pointer { refer, location } => {
                let refer = Type::specialize_dummy_type(upper, refer)?;
                Some(Type::Pointer {
                    refer: Box::new(refer),
                    location: location.clone(),
                })
            }
            Type::Array {
                content,
                length,
                location,
            } => {
                let content = Type::specialize_dummy_type(upper, content)?;
                Some(Type::Array {
                    content: Box::new(content),
                    length: *length,
                    location: location.clone(),
                })
            }
            Type::Any => None,
            Type::AnyRef => None,
            Type::Null => {
                if !Type::is_dummy_type(upper) {
                    match upper {
                        Type::Pointer { .. } => Some(upper.clone()),
                        Type::Array { content, .. } => {
                            let content = Type::specialize_dummy_type(content, lower)?;
                            Some(Type::Array {
                                content: Box::new(content),
                                length: None,
                                location: None,
                            })
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Type::Nothing => match upper {
                Type::Double(_)
                | Type::Float(_)
                | Type::Long(_)
                | Type::UnsignedLong(_)
                | Type::Int(_)
                | Type::UnsignedInt(_)
                | Type::Short(_)
                | Type::UnsignedShort(_)
                | Type::Byte => Some(Type::Byte),
                Type::Array { content, .. } => {
                    let content = Type::specialize_dummy_type(content, lower)?;
                    Some(Type::Array {
                        content: Box::new(content),
                        length: None,
                        location: None,
                    })
                }
                upper if !Type::is_dummy_type(upper) => Some(upper.clone()),
                _ => None,
            },
            _ => unreachable!(),
        }
    }

    /// Check whether a type is a dummy type.
    pub fn is_dummy_type(type_: &Type) -> bool {
        use Type::*;

        match type_ {
            Any | Nothing | AnyRef | Null => true,
            Byte => false, // Byte is promoted to Shoty in the end.
            Pointer { refer, .. } => Self::is_dummy_type(refer),
            Array { content, .. } => Self::is_dummy_type(content),
            _ => false,
        }
    }

    /// Determine the relationship of `left` type and `right` type.
    ///
    /// We treat `right` as the comparision base, which means if this function
    /// returns `Super`, then `left` is a supertype of `right`.
    pub fn compare_types(left: &Type, right: &Type) -> TypeRelationship {
        use Type::*;
        use TypeRelationship::*;

        // Check for unspecialized types, and extract specialized types.
        if let (T(None), _)
        | (_, T(None))
        | (Any, Any)
        | (Nothing, Nothing)
        | (AnyRef, AnyRef)
        | (Null, Null) = (left, right)
        {
            return Equal;
        }
        let left = &left.specialized().unwrap();
        let right = &right.specialized().unwrap();

        // All pointers (except `AnyRef` and `Null`) must be exactly
        // equal to the other types in assignments.
        // Arrays cannot be changed, but they can be assigned to a pointer.
        match (left, right) {
            (Pointer { refer: left, .. }, Pointer { refer: right, .. }) => {
                if left == right {
                    return Equal;
                } else {
                    return Invalid;
                }
            }
            (AnyRef, Pointer { .. }) | (Pointer { .. }, Null) => return Super,
            (Pointer { .. }, AnyRef) | (Null, Pointer { .. }) => return Sub,
            (Pointer { refer, .. }, Array { content, .. }) => {
                if refer == content {
                    return Equal;
                } else {
                    return Invalid;
                }
            }
            (Array { content: left, .. }, Array { content: right, .. }) => {
                // We allow this rule to enable array initialization.
                return Self::compare_types(left, right);
            }
            _ => (),
        }

        // Relationships involving `Any`, `Nothing`, `Void`, `Char`,
        // and `Structure` are trivially determined.
        match (left, right) {
            (Any, _) | (_, Nothing) => return Super,
            (_, Any) | (Nothing, _) => return Sub,
            (Void(_), Void(_)) | (Char(_), Char(_)) => return Equal,
            (Struct { name: left, .. }, Struct { name: right, .. }) => {
                if left == right {
                    return Equal;
                } else {
                    return Invalid;
                }
            }
            _ => (),
        }

        // Type relationships between numerical types.
        match (left, right) {
            (Double(..), Double(..)) => return Equal,
            (Double(..), Float(..)) => return Super,
            (Double(..), Long(..)) => return Super,
            (Double(..), UnsignedLong(..)) => return Super,
            (Double(..), Int(..)) => return Super,
            (Double(..), UnsignedInt(..)) => return Super,
            (Double(..), Short(..)) => return Super,
            (Double(..), UnsignedShort(..)) => return Super,
            (Double(..), Byte) => return Super,

            (Float(..), Double(..)) => return Sub,
            (Float(..), Float(..)) => return Equal,
            (Float(..), Long(..)) => return Super,
            (Float(..), UnsignedLong(..)) => return Super,
            (Float(..), Int(..)) => return Super,
            (Float(..), UnsignedInt(..)) => return Super,
            (Float(..), Short(..)) => return Super,
            (Float(..), UnsignedShort(..)) => return Super,
            (Float(..), Byte) => return Super,

            (Long(..), Double(..)) => return Sub,
            (Long(..), Float(..)) => return Sub,
            (Long(..), Long(..)) => return Equal,
            (Long(..), UnsignedLong(..)) => return Invalid,
            (Long(..), Int(..)) => return Super,
            (Long(..), UnsignedInt(..)) => return Super,
            (Long(..), Short(..)) => return Super,
            (Long(..), UnsignedShort(..)) => return Super,
            (Long(..), Byte) => return Super,

            (UnsignedLong(..), Double(..)) => return Sub,
            (UnsignedLong(..), Float(..)) => return Sub,
            (UnsignedLong(..), Long(..)) => return Invalid,
            (UnsignedLong(..), UnsignedLong(..)) => return Equal,
            (UnsignedLong(..), Int(..)) => return Invalid,
            (UnsignedLong(..), UnsignedInt(..)) => return Super,
            (UnsignedLong(..), Short(..)) => return Invalid,
            (UnsignedLong(..), UnsignedShort(..)) => return Super,
            (UnsignedLong(..), Byte) => return Super,

            (Int(..), Double(..)) => return Sub,
            (Int(..), Float(..)) => return Sub,
            (Int(..), Long(..)) => return Sub,
            (Int(..), UnsignedLong(..)) => return Invalid,
            (Int(..), Int(..)) => return Equal,
            (Int(..), UnsignedInt(..)) => return Invalid,
            (Int(..), Short(..)) => return Super,
            (Int(..), UnsignedShort(..)) => return Super,
            (Int(..), Byte) => return Super,

            (UnsignedInt(..), Double(..)) => return Sub,
            (UnsignedInt(..), Float(..)) => return Sub,
            (UnsignedInt(..), Long(..)) => return Invalid,
            (UnsignedInt(..), UnsignedLong(..)) => return Sub,
            (UnsignedInt(..), Int(..)) => return Invalid,
            (UnsignedInt(..), UnsignedInt(..)) => return Equal,
            (UnsignedInt(..), Short(..)) => return Invalid,
            (UnsignedInt(..), UnsignedShort(..)) => return Super,
            (UnsignedInt(..), Byte) => return Super,

            (Short(..), Double(..)) => return Sub,
            (Short(..), Float(..)) => return Sub,
            (Short(..), Long(..)) => return Sub,
            (Short(..), UnsignedLong(..)) => return Invalid,
            (Short(..), Int(..)) => return Sub,
            (Short(..), UnsignedInt(..)) => return Invalid,
            (Short(..), Short(..)) => return Equal,
            (Short(..), UnsignedShort(..)) => return Invalid,
            (Short(..), Byte) => return Super,

            (UnsignedShort(..), Double(..)) => return Sub,
            (UnsignedShort(..), Float(..)) => return Sub,
            (UnsignedShort(..), Long(..)) => return Invalid,
            (UnsignedShort(..), UnsignedLong(..)) => return Sub,
            (UnsignedShort(..), Int(..)) => return Invalid,
            (UnsignedShort(..), UnsignedInt(..)) => return Sub,
            (UnsignedShort(..), Short(..)) => return Invalid,
            (UnsignedShort(..), UnsignedShort(..)) => return Equal,
            (UnsignedShort(..), Byte) => return Super,

            (Byte, Double(..)) => return Sub,
            (Byte, Float(..)) => return Sub,
            (Byte, Long(..)) => return Sub,
            (Byte, UnsignedLong(..)) => return Sub,
            (Byte, Int(..)) => return Sub,
            (Byte, UnsignedInt(..)) => return Sub,
            (Byte, Short(..)) => return Sub,
            (Byte, UnsignedShort(..)) => return Sub,
            (Byte, Byte) => return Equal,

            _ => (),
        }

        // All other combinations are invalid.
        Invalid
    }
}

/// AST nodes for expressions.
#[derive(Debug, PartialEq)]
pub enum Expression {
    Ident {
        value: String,
        location: Location,
    },
    IntConst {
        value: i128,
        location: Location,
    },
    FloatConst {
        value: f64,
        location: Location,
    },
    CharConst {
        value: String,
        location: Location,
    },
    StrConst {
        value: String,
        location: Location,
    },
    Prefix {
        operator: &'static str,
        expression: Box<Expression>,
        location: Location,
    },
    Infix {
        left: Box<Expression>,
        operator: &'static str,
        right: Box<Expression>,
    },
    Postfix {
        operator: &'static str,
        expression: Box<Expression>,
    },
    Group {
        expression: Box<Expression>, // "(expression)"
        location: Location,
    },
    Index {
        expression: Box<Expression>,
        index: Box<Expression>,
    },
    Call {
        expression: Box<Expression>,
        arguments: Vec<Expression>,
    },
    InitList {
        initializers: Vec<(Option<String>, Expression)>, // { 1 } for arrays, { .mem = 1 } for structures
        location: Location,
    },
}

impl Locate for Expression {
    fn locate(&self) -> Location {
        use Expression::*;

        match self {
            Ident { location, .. }
            | IntConst { location, .. }
            | FloatConst { location, .. }
            | CharConst { location, .. }
            | StrConst { location, .. }
            | Prefix { location, .. }
            | Group { location, .. }
            | InitList { location, .. } => location.clone(),
            Infix { left, .. } => left.locate(),
            Postfix { expression, .. } => expression.locate(),
            Index { expression, .. } => expression.locate(),
            Call { expression, .. } => expression.locate(),
        }
    }
}

pub type Declarator = (Rc<RefCell<Type>>, String, Option<Expression>);

/// AST nodes for statements.
#[derive(Debug, PartialEq)]
pub enum Statement {
    Null(Location), // ";"
    Expr(Expression),
    Continue(Location),
    Break(Location),
    Include {
        content: String,
        location: Location,
    },
    Return {
        expression: Option<Expression>,
        location: Location,
    },
    Block {
        statements: Vec<Statement>,
        location: Location,
    },
    Def {
        base_type: Rc<RefCell<Type>>, // does not contain array/pointer definitions
        declarators: Vec<Declarator>,
        location: Location,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
        location: Location,
    },
    Do {
        condition: Expression,
        body: Box<Statement>,
        location: Location,
    },
    For {
        initialization: Option<Box<Statement>>,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Statement>,
        location: Location,
    },
    If {
        condition: Expression,
        body: Box<Statement>,
        alternative: Option<Box<Statement>>,
        location: Location,
    },
    Switch {
        expression: Expression,
        branches: Vec<(Expression, Vec<Statement>)>,
        default: Option<Vec<Statement>>,
        location: Location,
    },
}

impl Locate for Statement {
    fn locate(&self) -> Location {
        use Statement::*;

        match self {
            Null(loc) | Continue(loc) | Break(loc) => loc.clone(),
            Expr(expr) => expr.locate(),
            Return { location, .. }
            | Include { location, .. }
            | Block { location, .. }
            | Def { location, .. }
            | While { location, .. }
            | Do { location, .. }
            | For { location, .. }
            | If { location, .. }
            | Switch { location, .. } => location.clone(),
        }
    }
}

/// AST nodes for functions.
#[derive(Debug, PartialEq)]
pub struct Function {
    pub is_proto: bool, // whether it is a prototype for internal usages only
    pub return_type: Rc<RefCell<Type>>,
    pub name: String,
    pub parameters: IndexMap<String, Rc<RefCell<Type>>>,
    pub ellipsis: bool, // whether the parameter list contains an ellipsis
    pub body: Statement,
    pub location: Location,
}

impl Locate for Function {
    fn locate(&self) -> Location {
        self.location.clone()
    }
}

/// AST nodes for static objects that can appear in the global scope.
#[derive(Debug, PartialEq)]
pub enum StaticObject {
    Type(Type),              // structures
    Statement(Statement),    // #include
    Function(Box<Function>), // Boxing the large field to reduce the total size of the enum.
}
