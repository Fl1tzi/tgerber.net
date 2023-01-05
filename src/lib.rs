use wasm_bindgen::prelude::*;
use web_sys;
use yew::html::Scope;
use yew::prelude::*;
mod post_data;

#[wasm_bindgen]
extern "C" {
    #[derive(Debug)]
    type Buffer;
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u32(a: u32);
}

// console log
#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[derive(Properties, PartialEq, Default)]
pub struct PostProps {
    #[prop_or_default]
    pub children: Children,
}

pub struct App {
    pub container: NodeRef,
    pub post_prompt_text: &'static str,
    pub post_prompt_title: &'static str,
    pub post_prompt_hash: usize,
}

pub enum Msg {
    // this uses the post number (useful for search)
    // why? allows me to manually set index and change out items without letting them change
    OpenBox(usize),
    // this uses the array index (useful for buttons)
    // why? It's probably more efficient
    OpenBoxIndex(usize),
    OpenError(ErrorType),
    CloseBox,
}

pub enum ErrorType {
    NotFound,
    ParseError,
    IndexNotFound,
}

impl App {
    // TODO: make it smaller because it almost the same
    fn open_box(&self, _sink: &Scope<Self>) {
        console_log!("[  ] Opening post");
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container_all = document.get_element_by_id("container-all").unwrap();
        let box_element = document.get_element_by_id("post-popup-box").unwrap();
        let box_inner = document.get_element_by_id("post-popup-inner").unwrap();
        container_all
            .set_attribute("style", "display: none;")
            .expect("Could not hide main container");
        box_element
            .set_attribute("style", "display: block;")
            .expect("Could not show outer box");
        box_inner
            .set_attribute("style", "display: block;")
            .expect("Could not show inner box");
    }
    fn close_box(&self, _sink: &Scope<Self>) {
        console_log!("[X] Closing post");
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let container_all = document.get_element_by_id("container-all").unwrap();
        let box_element = document.get_element_by_id("post-popup-box").unwrap();
        let box_inner = document.get_element_by_id("post-popup-inner").unwrap();
        container_all
            .set_attribute("style", "display: block;")
            .expect("Could not show the main container");
        box_element
            .set_attribute("style", "display: none;")
            .expect("Could not hide outer box");
        box_inner
            .set_attribute("style", "display: none;")
            .expect("Could not hide inner box");
        // scrolling back into the view
        match document.get_element_by_id(format!("post-{}", self.post_prompt_hash).as_str()) {
            Some(pe) => pe.scroll_into_view(),
            None => console_log!("Cannot scroll to post (does not exist)"),
        }
    }
}

fn get_post_index(n: usize) -> Option<usize> {
    post_data::POSTS.iter().position(|p| p.number == n)
}

impl Component for App {
    type Message = Msg;
    type Properties = PostProps;

    fn create(_ctx: &Context<Self>) -> Self {
        let document = gloo_utils::document();
        let url = document.url().unwrap();
        let vals = url.split("/").collect::<Vec<&str>>();
        let index_str = vals[vals.len() - 1];
        if index_str != "" {
            match index_str.parse::<usize>() {
                Ok(index) => {
                    let mut found: bool = false;
                    // see if there is a post with that index
                    for post in post_data::POSTS {
                        if post.number == index {
                            found = true;
                        }
                    }
                    if found == false {
                        // there is no post with that index
                        _ctx.link()
                            .callback(move |_| Msg::OpenError(ErrorType::NotFound))
                            .emit(());
                    } else {
                        // there is a post with that index
                        _ctx.link()
                            .callback(move |v: usize| Msg::OpenBox(v))
                            .emit(index);
                    }
                }
                Err(_e) => {
                    _ctx.link()
                        .callback(move |_| Msg::OpenError(ErrorType::ParseError))
                        .emit(());
                }
            }
        }
        Self {
            container: NodeRef::default(),
            post_prompt_text: "",
            post_prompt_title: "",
            post_prompt_hash: 0,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        //let container = self
        //    .container;
        // let container = Html::VRef(container.into());

        match msg {
            Msg::OpenBoxIndex(n) => {
                if n >= post_data::POSTS.len() {
                    _ctx.link()
                        .callback(move |_| Msg::OpenError(ErrorType::IndexNotFound))
                        .emit(());
                    false
                } else {
                    let post = &post_data::POSTS[n];
                    self.post_prompt_text = post.desc;
                    self.post_prompt_title = post.name;
                    self.post_prompt_hash = post.number;
                    self.open_box(_ctx.link());
                    let location = gloo_utils::window().location();
                    location
                        .set_href(format!("/#/{}", post.number).as_str())
                        .unwrap();
                    true
                }
            }
            Msg::OpenBox(n) => {
                console_log!("Opening box");
                // let post = &POSTS[n];
                match get_post_index(n) {
                    Some(index) => {
                        let post = &post_data::POSTS[index];
                        self.post_prompt_text = post.desc;
                        self.post_prompt_title = post.name;
                        self.post_prompt_hash = post.number;
                        self.open_box(_ctx.link());
                        true
                    }
                    None => {
                        _ctx.link()
                            .callback(move |_| Msg::OpenError(ErrorType::IndexNotFound))
                            .emit(());
                        false
                    }
                }
            }
            Msg::CloseBox => {
                // console_log!("Closing box");
                let location = gloo_utils::window().location();
                location.set_href("/#/").unwrap();
                self.close_box(_ctx.link());
                true
            }
            Msg::OpenError(e) => {
                // console_log!("Showing error");
                self.post_prompt_title = "Error";
                // I will probably shoot me into my own foot with that lol
                self.post_prompt_hash = 404;
                self.post_prompt_text = match e {
                    ErrorType::NotFound => "<p>The post you were searching for was not found.</p>",
                    ErrorType::ParseError => "<p>Search value not allowed. The argument has to be a number being positive.</p>",
                    ErrorType::IndexNotFound => "<p>Internal error: The requested index was not found.</p><p>If this issue persists please open an issue on https://github.com/Fl1tzi/tgerber.net</p>"
                };
                self.open_box(_ctx.link());
                true
            }
        }
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        let document = gloo_utils::document();
        let get_rendered_html = |html| -> yew::virtual_dom::VNode {
            let element = document.create_element("div").unwrap();
            element.set_inner_html(html);
            Html::VRef(element.into())
        };
        html! {
        <>
        // popup
        <div class="post-popup-box" id="post-popup-box">
            <div class="post-popup-inner" id="post-popup-inner">
                <button class="no-btn-style" style="float: right;" onclick={ _ctx.link().callback(|_| Msg::CloseBox) }>
                <span class="close-popup">{ "[X]" }</span>
                </button>
                <span style="font-size: 30px;">{ format!("// {} ", &self.post_prompt_title) }</span>
                <span style="color: var(--color-full);">{
                    format!("#{}", &self.post_prompt_hash) }
                </span>
                <br/>
                { get_rendered_html(&self.post_prompt_text) }
            </div>
        </div>
        // the container for all items that are not the message
        <div id="container-all">
        // socials
        <div class="container">
            <div class="logo">
                <h1 class="section-title">{ "Fl1tzi" }</h1>

                <div>
                    <p class="section-subtitle-name">{ "Information" }</p>
                    <p class="section-subtitle">{ "These are ways to contact me. Fastest way is E-Mail or Matrix." }</p>
                </div>
            </div>
            <div class="grid">
                <div class="card-social">
                    <div class="inner-social">
                        <span class="c-title">{ "E-Mail" }</span>
                        <img src="assets/mail-fill.svg" alt="mail icon" class="c-logo"/>
                        <br/>
                        // <img src="http://ghchart.rshah.org/Fl1tzi" alt="Github chart" style="margin-top: 20px; margin-bottom: 10px; width: 90%;"/>
                        <a class="no-underline" href="mailto:kontakt@tgerber.net">
                        <button class="btn">{ "kontakt@tgerber.net" }</button>
                        </a>
                    </div>
                </div>
                <div class="card-social">
                    <div class="inner-social">
                        <span class="c-title">{ "GitHub" }</span>
                        <img src="assets/GitHub.png" alt="GitHub avatar" class="c-logo"/>
                        <br/>
                        // <img src="http://ghchart.rshah.org/Fl1tzi" alt="Github chart" style="margin-top: 20px; margin-bottom: 10px; width: 90%;"/>
                        <a class="no-underline" href="https://github.com/Fl1tzi">
                        <button class="btn">{ "Fl1tzi" }</button>
                        </a>
                    </div>
                </div>
                <div class="card-social">
                    <div class="inner-social">
                        <span class="c-title">{ "Discord" }</span>
                        <img src="assets/Discord.png" alt="Discord avatar" class="c-logo"/>
                        <br/>
                        <p>{ "Fl1tzi#0001" }</p>
                    </div>
                </div>
                <div class="card-social">
                    <div class="inner-social">
                        <span class="c-title">{ "Matrix" }</span>
                        <span style="font-size: 25px;">{ "[ Matrix ]" }</span>
                        <br/>
                        <a class="no-underline" href="https://matrix.to/#/@fl1tzi:matrix.fl1tzi.com">
                        <button class="btn" style="margin-top: 35px;">{ "@Fl1tzi:matrix.fl1tzi.com" }</button>
                        </a>
                    </div>
                </div>
            </div>
         </div>

         <div class="container" id="post-container">
            <div class="logo">
                <h1 class="section-title">{ "Posts" }</h1>
            </div>

            <div class="grid">
                { for post_data::POSTS.iter().enumerate().map(|(index, post)| {
                                              html! {
                                                  <div class="card-post" id={ format!("post-{}", post.number) }>
                                                    <div class="tc-container">
                                                        <span class="c-title">{
                                                            format!("#{}",
                                                                    post.number
                                                                    )
                                                            }
                                                        </span>
                                                        <h3 class="post-title">
                                                            { post.name }
                                                        </h3>
                                                        <p class="post-prev">
                                                            { post.prev }
                                                        </p>
                                                    </div>

                                                    <div class="bc-container">
                                                        <button onclick={_ctx
                                                            .link()
                                                            .callback(move |_| Msg::OpenBoxIndex(index))} class="btn">{ "Open post" }</button>
                                                          </div>
                                                      </div>
                                              }
                                          }) }

            </div>
        </div>

        <div>
        <footer>
            <details style="margin-top: 40px; font-size: 13px; word-break: break-word;">
                // I can't get it to center properly
                <summary style="border: 1px solid var(--color-full); padding: 10px;">
                <p>{ "Datenschutzerklärung" }</p>
                </summary>
                <code>
                <p>{"
                Kontakt: kontakt@tgerber.net
                "}</p>
                <p>{"
                Ich verwende für diese Webseite einen Webhosting-Dienst von GitHub Inc. (GitHub Pages).Unser Hoster erhebt in sog. Logfiles folgende Daten, die Ihr Browser übermittelt:

IP-Adresse, die Adresse der vorher besuchten Website (Referer Anfrage-Header), Datum und Uhrzeit der Anfrage, Zeitzonendifferenz zur Greenwich Mean Time, Inhalt der Anforderung, HTTP-Statuscode, übertragene Datenmenge, Website, von der die Anforderung kommt und Informationen zu Browser und Betriebssystem.
                "}</p>

                <p>{"Wir setzen für die Zurverfügungstellung unserer Website folgenden Hoster ein:"}</p>

                <em><p>{"
                GitHub Inc.:
                88 Colin P. Kelly Jr. St.
                San Francisco
                CA 94107 USA
                "}</p></em>

                <p>{ "Dieser ist Empfänger Ihrer personenbezogenen Daten. Dies entspricht unserem berechtigten Interesse im Sinne des Art. 6 Abs. 1 S. 1 lit. f DSGVO, selbst keinen Server in unseren Räumlichkeiten vorhalten zu müssen. Serverstandort ist USA." }</p>
                <p>{"
                Es erfolgt kein Tracking und wir haben auf diese Daten keinen direkten Zugriff, sondern erhalten lediglich eine anonymisierte, statistische Zusammenfassung. Diese beinhaltet die Adresse der vorher besuchten Seite, die Häufigkeit der jeweils aufgerufenen Seiten und die Anzahl eindeutiger Besucher. Diese Daten führen wir nicht mit anderen Daten zusammen.
                "}</p>

                <p>{"
                Weitere Informationen zu Widerspruchs- und Beseitigungsmöglichkeiten gegenüber GitHub finden Sie unter: https://docs.github.com/en/free-pro-team@latest/github/site-policy/github-privacy-statement#github-pages
                    "}</p>

                    <p>{"
                    GitHub hat Compliance-Maßnahmen für internationale Datenübermittlungen umgesetzt. Diese gelten für alle weltweiten Aktivitäten, bei denen GitHub personenbezogene Daten von natürlichen Personen in der EU verarbeitet. Diese Maßnahmen basieren auf den EU-Standardvertragsklauseln (SCCs). Weitere Informationen finden Sie unter: https://docs.github.com/en/free-pro-team@latest/github/site-policy/github-data-protection-addendum#attachment-1–the-standard-contractual-clauses-processors
                    "}</p>

                    <p>{ "Rechtliche Hinweise" }</p>

                    <p>{"
Grundsätzlich ist ein Auftragsverarbeitungsvertrag mit dem Hoster abzuschließen. Das bayerische Landesamt für Datenschutzaufsicht hat für das Hosting rein statischer Websites eine Ausnahme gemacht. Für den Fall, dass die Webseite der Selbstdarstellung dient, z.B. von Vereinen oder Kleinunternehmen, keine personenbezogenen Daten an den Betreiber fließen und kein Tracking stattfindet, liegt keine Auftragsverarbeitung vor. Weiter heißt es: „Die Tatsache, dass auch beim Hosting von statischen Webseiten zwangsläufig IP-Adressen, d.h. personenbezogene Daten, verarbeitet werden müssen, führt nicht zur Annahme einer Auftragsverarbeitung. Das wäre nicht sachgerecht. Die (kurzfristige) IP-Adressenspeicherung ist vielmehr noch der TK-Zugangsvermittlung des Website-Hosters nach dem TKG zuzurechnen und dient in erster Linie Sicherheitszwecken des Hosters.“ (https://www.lda.bayern.de/media/veroeffentlichungen/FAQ_Hosting_keine_Auftragsverarbeitung.pdf) Wir gehen davon aus, dass diese Ausnahme auf GitHub Pages anzuwenden ist.

                    "}</p>

                <p>{"
                GitHub Datenverarbeitungsbedingungen: https://docs.github.com/en/site-policy/privacy-policies/github-data-protection-agreement"}</p>
                <p>{"
                GitHub Datenschutzerklärung: https://docs.github.com/en/site-policy/privacy-policies/github-privacy-statement?tid=134222759
                "}</p>
            </code>
            </details>
            <p>{ "//" }</p>
        <p>{ "short version: This site does not use any data from you but GitHub (the hoster) could save some data." }</p>
        <p style="font-size: 12px;">{ "Thank you for visiting!" }</p>
        </footer>
        </div>
        </div>
        </>
        }
    }
}

/* #[function_component]
fn App() -> Html {
    let _err_text = |e: io::Error| -> Vec<Html> {
        vec![html_nested! { <p style="color: red;"> {
            format!("Error listing posts: {}", e) } </p>
        }]
    };

    let _window = web_sys::window();
    html!{}

}*/

#[wasm_bindgen(start)]
pub fn main() {
    yew::Renderer::<App>::new().render();
}
