import sys

def define_ast(out_dir, base_name, types):
    path  = out_dir + "/" + base_name.lower() + "/mod.rs"
    with open(path, "w") as f:
        f.write("pub mod compiler;\npub mod parser;\n\n")
        f.write("use super::token::{Token, literal::Literal};\nuse std::fmt::Debug;\n\n")
        f.write("#[derive(Clone, Debug)]\npub enum " + base_name + " {\n")

        for _type in types:
            type_name = _type.split("|")[0].strip()
            f.write("    " + type_name + "(")
            field_list = _type.split("|")[1].strip()
            fields = field_list.split(", ")
            if fields[0] == "Expr":
                    fields[0] = "Box<Expr>"
            f.write(fields[0])
            for field in fields[1:]:
                f.write(", ")
                if field == base_name:
                    field = "Box<" + base_name + ">"
                f.write(field)
            f.write("),\n")

        f.write("}")

        #

        # f.write("impl " + base_name + " {\n")
        # for _type in types:
        #     type_name = _type.split("|")[0].strip()
        #     field_list = _type.split("|")[1].strip()
        #     fields = field_list.split(", ")

        #     # f.write("pub struct " + type_name + " {\n")
        #     # for field in fields:
        #     #     f.write("    "+field + ",\n")
        #     # f.write("}\n\n")

        #     # f.write("impl " + type_name + " {\n")
        #     f.write("    pub fn new_" + type_name.lower() + "(")
        #     f.write("_0: " + fields[0])
        #     for i in range(1, len(fields)):
        #         f.write(", _" + str(i) + ": " + fields[i])
        #     f.write(") -> " + base_name + " {\n")
        #     f.write("        " + base_name + "::" + type_name + "(")
        #     if fields[0] == "Expr":
        #         f.write("Box::new(_0)")
        #     else:
        #         f.write("_0")
        #     for i in range(1, len(fields)):
        #         if fields[i] == "Expr":
        #             f.write(", Box::new(_" + str(i) + ")")
        #         else:
        #             f.write(", _" + str(i))
        #     f.write(")\n")
        #     # for field in fields:
        #     #     f.write("            " + field + ",\n")
        #     f.write("    }\n")
        # f.write("}\n")
    

if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise Exception("Usage: generate_ast <output directory>")

    out_dir: str = sys.argv[1]
    define_ast(out_dir, "Expr", [
        "Binary | Expr, Token, Expr",
        "MsgEmission | Option<Box<Expr>>, Token, Option<Box<Expr>>",
        "BinaryOpt | Expr, Token, Option<Box<Expr>>",
        "Asm | Expr, Expr",
        "Object | Vec<Expr>",
        "Fn | Vec<Expr>, Expr",
        "CodeBlock | Vec<Expr>",
        "Type | Vec<Expr>",
        "Literal | Literal"
    ])