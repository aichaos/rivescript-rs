#[cfg(test)]
mod tests {
    use crate::parser::Parser;

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[test]
    fn test_parser_ast() {
        let parser = Parser::new();
        let source = String::from(
            r"
            ! version = 2.0

            ! global depth = 64
            ! global debug = true

            ! array colors = red blue green
            ^ yellow cyan magenta
            ^ light\sblue light\sred
            ^ light green|light yellow

            ! local concat = newline

            ! var name = RiveScript
            ^ Test Robot

            + ask me a question
            - What is your name?

            + *
            % what is your name
            - <set name=<formal>>I will call you <get name>.
            ",
        );
        let ast = parser.parse("test", source).unwrap();
        assert_eq!(ast.version, 2.0);
        assert_eq!(ast.globals.get("depth").unwrap(), "64");
        assert_eq!(ast.globals.get("debug").unwrap(), "true");
        assert_eq!(ast.vars.get("name").unwrap(), "RiveScriptTest Robot");

        // Ensure the array parsed in correctly.
        let actual_array = ast.arrays.get("colors").unwrap();
        let expect_array: Vec<String> = vec![
            "red",
            "blue",
            "green",
            "yellow",
            "cyan",
            "magenta",
            "light blue",
            "light red",
            "light green",
            "light yellow",
        ]
        .into_iter()
        .map(|s| s.to_owned())
        .collect();

        assert!(expect_array.iter().all(|item| actual_array.contains(item)));
        assert!(actual_array.iter().all(|item| expect_array.contains(item)));
    }
}
