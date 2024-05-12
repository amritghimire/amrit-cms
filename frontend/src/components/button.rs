use dioxus::prelude::*;

#[derive(PartialEq, Props, Clone)]
pub struct ButtonProps {
    progress: bool,
    #[props(into)]
    class: String,
    #[props(extends = button)]
    attributes: Vec<Attribute>,
    children: Element,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            class: props.class,
            class: "flex disabled:bg-neutral-600 h-10",
            disabled: props.progress,
            ..props.attributes,
            if props.progress {
                svg {
                    "fill": "none",
                    "xmlns": "http://www.w3.org/2000/svg",
                    "viewBox": "0 0 24 24",
                    class: "animate-spin -ml-1 mr-3 h-5 w-5 text-white",
                    circle {
                        "r": "10",
                        "cx": "12",
                        "stroke": "currentColor",
                        "stroke-width": "4",
                        "cy": "12",
                        class: "opacity-25"
                    }
                    path {
                        "d": "M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z",
                        "fill": "currentColor",
                        class: "opacity-75"
                    }
                }
            }
            {props.children}
        }
    }
}
