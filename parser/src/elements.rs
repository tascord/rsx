use dominator::DomBuilder;
use macros::{json_enum, json_match};

json_enum!(Elements, "./parser/mdn/element.json",
    v =>
        Object
        .keys(v)
        .flatMap(v => v
            .split(',')
            .map(v => v.trim())
        )
        .map(v => v
            .replace(/[<>]/g, "")
        )
);
