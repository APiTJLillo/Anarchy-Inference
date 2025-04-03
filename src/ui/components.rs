use yew::prelude::*;
use crate::interpreter::Interpreter;
use crate::parser::Parser;
use crate::lexer::Lexer;

#[derive(Properties, PartialEq)]
pub struct WindowProps {
    pub title: String,
    pub children: Children
}

#[function_component(Window)]
pub fn window(props: &WindowProps) -> Html {
    html! {
        <div class="window">
            <div class="title-bar">{ &props.title }</div>
            <div class="window-content">
                { props.children.clone() }
            </div>
        </div>
    }
}

#[derive(Properties, PartialEq)]
pub struct TextProps {
    pub content: String
}

#[function_component(Text)]
pub fn text(props: &TextProps) -> Html {
    html! {
        <div class="text">{ &props.content }</div>
    }
}

#[derive(Properties, PartialEq)]
pub struct ButtonProps {
    pub label: String,
    pub onclick: Callback<MouseEvent>
}

#[function_component(Button)]
pub fn button(props: &ButtonProps) -> Html {
    html! {
        <button onclick={props.onclick.clone()}>
            { &props.label }
        </button>
    }
}

#[derive(Properties, PartialEq)]
pub struct InputProps {
    pub value: String,
    pub onchange: Callback<Event>,
    pub placeholder: String
}

#[function_component(Input)]
pub fn input(props: &InputProps) -> Html {
    html! {
        <input
            type="text"
            placeholder={props.placeholder.clone()}
            value={props.value.clone()}
            onchange={props.onchange.clone()}
        />
    }
}

#[function_component(Editor)]
pub fn editor() -> Html {
    let input_value = use_state(|| String::new());
    let output_value = use_state(|| String::new());

    let onchange = {
        let input_value = input_value.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            input_value.set(target.value());
        })
    };

    let onclick = {
        let input_value = input_value.clone();
        let output_value = output_value.clone();
        
        Callback::from(move |_| {
            let code = (*input_value).clone();
            let mut lexer = Lexer::new(code.clone());
            let tokens = lexer.tokenize().unwrap_or_default();
            let mut parser = Parser::new(tokens);
            let mut interpreter = Interpreter::new();
            
            match parser.parse() {
                Ok(ast_nodes) => {
                    // Use the first node from the returned Vec<ASTNode>
                    if let Some(ast) = ast_nodes.first() {
                        match interpreter.execute(ast) {
                            Ok(result) => {
                                output_value.set(format!("Result: {:?}", result));
                            }
                            Err(e) => {
                                output_value.set(format!("Runtime error: {:?}", e));
                            }
                        }
                    } else {
                        output_value.set("No AST nodes generated".to_string());
                    }
                }
                Err(e) => {
                    output_value.set(format!("Parse error: {:?}", e));
                }
            }
        })
    };

    html! {
        <Window title="Anarchy Inference IDE">
            <div class="editor">
                <Input
                    value={(*input_value).clone()}
                    onchange={onchange}
                    placeholder="Enter your code..."
                />
                <Button label="Run" {onclick} />
                <Text content={(*output_value).clone()} />
            </div>
        </Window>
    }
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div>
            <Editor />
        </div>
    }
}
