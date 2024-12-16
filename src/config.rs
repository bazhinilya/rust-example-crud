use actix_web::web;

use crate::handler::{
    create_note_handler, delete_note_handler, edit_note_handler, get_note_handler,
    note_list_handler,
};

pub fn config(conf: &mut web::ServiceConfig) {
    let scope = web::scope("/api")
        .service(note_list_handler)
        .service(create_note_handler)
        .service(get_note_handler)
        .service(edit_note_handler)
        .service(delete_note_handler);

    conf.service(scope);
}
