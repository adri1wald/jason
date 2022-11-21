pub mod ast;
pub mod lexer;
pub mod parser;

#[cfg(test)]
mod tests {
    use super::parser::parse_json;

    #[test]
    fn it_parses() {
        let input = "{\"root\":{\"children\":[{\"children\":[{\"children\":[{\"detail\":0,\"format\":0,\"mode\":\"normal\",\"style\":\"\",\"text\":\"One\",\"type\":\"text\",\"version\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"listitem\",\"version\":1,\"value\":1},{\"children\":[{\"children\":[{\"children\":[{\"children\":[{\"children\":[{\"detail\":0,\"format\":0,\"mode\":\"normal\",\"style\":\"\",\"text\":\"Two\",\"type\":\"text\",\"version\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":2,\"type\":\"listitem\",\"version\":1,\"value\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"list\",\"version\":1,\"listType\":\"bullet\",\"start\":1,\"tag\":\"ul\"}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":1,\"type\":\"listitem\",\"version\":1,\"value\":1},{\"children\":[{\"detail\":0,\"format\":0,\"mode\":\"normal\",\"style\":\"\",\"text\":\"Haha\",\"type\":\"text\",\"version\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":1,\"type\":\"listitem\",\"version\":1,\"value\":1}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"list\",\"version\":1,\"listType\":\"number\",\"start\":1,\"tag\":\"ol\"}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"listitem\",\"version\":1,\"value\":2}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"list\",\"version\":1,\"listType\":\"bullet\",\"start\":1,\"tag\":\"ul\"}],\"direction\":\"ltr\",\"format\":\"\",\"indent\":0,\"type\":\"root\",\"version\":1}}";
        let node = parse_json(input);
        dbg!(node);
        assert!(false);
    }
}
