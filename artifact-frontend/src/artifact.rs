use dev_prelude::*;


pub(crate) fn view_artifact(model: &Model, name: &Name) -> HtmlApp {
    match model.shared.artifacts.get(name) {
        Some(ref art) => {
            html! [
                <div><h1 class=H1,>{ name }</h1></div>
                <div class=(CLEARFIX, PY1),>
                    <div class=(COL, COL_3),>{ "comp?" }</div>
                    <div class=(COL, COL_3),>{ "test?" }</div>
                    <div class=(COL, COL_6),>{ "done?" }</div>
                </div>
                <div><textarea readonly=true, cols=80, rows=100,>
                    { &art.text }
                </textarea></div>
            ]
        },
        None => return html![
            <h3 class=H3,>{format!("Artifact with name {:?} not found", name)}</h3>
        ],
    }

    // div [ class "clearfix py1" ]
    //     [ div [ class "col col-6" ] (viewCompletedPerc artifact)
    //     , div [ class "col col-6" ] (viewTestedPerc artifact)
    //     ]
}

trait CompletedExt {
    fn spc_style(&self) -> HtmlApp;
    fn tst_style(&self) -> HtmlApp;
    fn name_color(&self) -> &'static str;
}

impl CompletedExt for Completed {
    /// #SPC-cli-ls.color_spc
    fn spc_style(&self) -> HtmlApp {
        let color = match self.spc_points() {
            0 => RED,
            1 => YELLOW,
            2 => BLUE,
            3 => OLIVE,
            _ => unreachable!(),
        };
        html![
            // <span color=color,>
            //     format!("{:.1}", self.spc * 100.0)
            // </span>
        ]
    }

    /// #SPC-cli-ls.color_tst
    fn tst_style(&self) -> HtmlApp {
        let color = match self.tst_points() {
            0 => RED,
            1 => YELLOW,
            2 => OLIVE,
            _ => unreachable!(),
        };
        html![
            // <span color=color,>
            //     format!("{:.1}", self.tst * 100.0)
            // </span>
        ]
    }



    /// #SPC-cli-ls.color_name
    fn name_color(&self) -> &'static str {
        match self.spc_points() + self.tst_points() {
            0 => RED,
            1 | 2 => YELLOW,
            3 | 4 => BLUE,
            5 => OLIVE,
            _ => unreachable!(),
        }
    }
}
