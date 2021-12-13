use super::*;
use crate::{token::Location, typed_ast::Variable};

#[derive(Debug, Clone, Copy)]
struct ClassDef<'a> {
    constructors: &'a [(&'a str, R, &'a [P])],
    methods: &'a [(&'a str, R, &'a [P])],
    functions: &'a [(&'a str, R, &'a [P])],
}

const CLASSES: &[(&str, ClassDef)] = &[
    (
        "Math",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("abs", R::Int, &[P::Int("x")]),
                ("multiply", R::Int, &[P::Int("x"), P::Int("y")]),
                ("divide", R::Int, &[P::Int("x"), P::Int("y")]),
                ("min", R::Int, &[P::Int("x"), P::Int("y")]),
                ("max", R::Int, &[P::Int("x"), P::Int("y")]),
                ("sqrt", R::Int, &[P::Int("x")]),
            ],
        },
    ),
    (
        "String",
        ClassDef {
            constructors: &[("new", R::String, &[P::Int("maxLength")])],
            methods: &[
                ("dispose", R::Void, &[]),
                ("length", R::Int, &[]),
                ("charAt", R::Char, &[P::Int("j")]),
                ("setCharAt", R::Void, &[P::Int("j"), P::Char("c")]),
                ("appendChar", R::String, &[P::Char("c")]),
                ("eraseLastChar", R::Void, &[]),
                ("intValue", R::Int, &[]),
                ("setInt", R::Void, &[P::Int("j")]),
            ],
            functions: &[
                ("backSpace", R::Char, &[]),
                ("doubleQuote", R::Char, &[]),
                ("newLine", R::Char, &[]),
            ],
        },
    ),
    (
        "Array",
        ClassDef {
            constructors: &[("new", R::Array, &[P::Int("size")])],
            methods: &[("dispose", R::Void, &[])],
            functions: &[],
        },
    ),
    (
        "Output",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("moveCursor", R::Void, &[P::Int("i"), P::Int("j")]),
                ("printChar", R::Void, &[P::Char("c")]),
                ("printString", R::Void, &[P::String("s")]),
                ("printInt", R::Void, &[P::Int("i")]),
                ("println", R::Void, &[]),
                ("backSpace", R::Void, &[]),
            ],
        },
    ),
    (
        "Screen",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("clearScreen", R::Void, &[]),
                ("setColor", R::Void, &[P::Boolean("b")]),
                ("drawPixel", R::Void, &[P::Int("x"), P::Int("y")]),
                (
                    "drawLine",
                    R::Void,
                    &[P::Int("x1"), P::Int("y1"), P::Int("x2"), P::Int("y2")],
                ),
                (
                    "drawRectangle",
                    R::Void,
                    &[P::Int("x1"), P::Int("y1"), P::Int("x2"), P::Int("y2")],
                ),
                (
                    "drawCircle",
                    R::Void,
                    &[P::Int("x"), P::Int("y"), P::Int("r")],
                ),
            ],
        },
    ),
    (
        "Keyboard",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("keyPressed", R::Char, &[]),
                ("readChar", R::Char, &[]),
                ("readLine", R::String, &[P::String("message")]),
                ("readInt", R::Int, &[P::String("message")]),
            ],
        },
    ),
    (
        "Memory",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("peek", R::Int, &[P::Int("address")]),
                ("poke", R::Void, &[P::Int("address"), P::Int("value")]),
                ("alloc", R::Array, &[P::Int("size")]),
                ("deAlloc", R::Void, &[P::Array("o")]),
            ],
        },
    ),
    (
        "Sys",
        ClassDef {
            constructors: &[],
            methods: &[],
            functions: &[
                ("init", R::Void, &[]),
                ("halt", R::Void, &[]),
                ("error", R::Void, &[P::Int("errorCode")]),
                ("wait", R::Void, &[P::Int("duration")]),
            ],
        },
    ),
];

impl GlobalSymbolTable {
    pub fn with_builtin() -> Self {
        let mut table = HashMap::new();
        for (name, def) in CLASSES {
            def.register(name, &mut table);
        }
        Self { table }
    }
}

fn conv_defs<'a, T>(
    defs: &'a [(&str, R, &'a [P])],
    f: impl Fn(WithLoc<Ident>, WithLoc<ReturnType>, Vec<WithLoc<Variable>>) -> T + 'a,
) -> impl Iterator<Item = (Ident, T)> + 'a {
    defs.iter().map(move |(name, return_type, params)| {
        let key = Ident::new(*name);
        let name = loc(Ident::new(*name));
        let return_type = loc(ReturnType::from(*return_type));
        let params = params.iter().map(|p| loc(Variable::from(*p))).collect();
        (key, f(name, return_type, params))
    })
}

impl ClassDef<'_> {
    fn methods(&self) -> impl Iterator<Item = (Ident, Method)> + '_ {
        conv_defs(self.methods, |name, return_type, params| Method {
            name,
            return_type,
            params,
        })
    }

    fn constructors(&self) -> impl Iterator<Item = (Ident, ClassMethod)> + '_ {
        conv_defs(self.constructors, |name, return_type, params| ClassMethod {
            name,
            kind: ClassMethodKind::Constructor,
            return_type,
            params,
        })
    }

    fn functions(&self) -> impl Iterator<Item = (Ident, ClassMethod)> + '_ {
        conv_defs(self.functions, |name, return_type, params| ClassMethod {
            name,
            kind: ClassMethodKind::Function,
            return_type,
            params,
        })
    }

    fn register(&self, class_name: &str, table: &mut HashMap<Ident, Symbol>) {
        let key = Ident::new(class_name);
        let class_name = loc(Ident::new(class_name));
        let path = PathBuf::from("<builtin>");

        let mut methods = HashMap::new();
        let mut class_methods = HashMap::new();

        methods.extend(self.methods());
        class_methods.extend(self.constructors());
        class_methods.extend(self.functions());

        table.insert(
            key,
            Symbol::Class(ExternalClassSymbolTable {
                class_name,
                path,
                methods,
                class_methods,
            }),
        );
    }
}

const fn loc<T>(data: T) -> WithLoc<T> {
    WithLoc {
        data,
        loc: Location::builtin(),
    }
}

fn ty_string() -> Type {
    Type::Class(Ident::new("String"))
}

fn ty_array() -> Type {
    Type::Class(Ident::new("Array"))
}

#[derive(Debug, Clone, Copy)]
enum R {
    Void,
    Int,
    Char,
    String,
    Array,
}

impl From<R> for ReturnType {
    fn from(r: R) -> Self {
        match r {
            R::Void => ReturnType::Void,
            R::Int => ReturnType::Type(loc(Type::Int)),
            R::Char => ReturnType::Type(loc(Type::Char)),
            R::String => ReturnType::Type(loc(ty_string())),
            R::Array => ReturnType::Type(loc(ty_array())),
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum P {
    Int(&'static str),
    Char(&'static str),
    Boolean(&'static str),
    String(&'static str),
    Array(&'static str),
}

impl From<P> for Variable {
    fn from(p: P) -> Self {
        let (ty, name) = match p {
            P::Int(name) => (Type::Int, name),
            P::Char(name) => (Type::Char, name),
            P::Boolean(name) => (Type::Boolean, name),
            P::String(name) => (ty_string(), name),
            P::Array(name) => (ty_array(), name),
        };
        Variable {
            name: loc(Ident::new(name)),
            ty: loc(ty),
        }
    }
}
