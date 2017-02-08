use aqua_web::plug;
use time::precise_time_ns;

pub fn plug(conn: &mut plug::Conn) {
    let start_time = precise_time_ns();
    conn.register_before_send(move |_conn: &mut plug::Conn| {
        let delta = precise_time_ns() - start_time;
        info!("request took {}ms", delta as f64 / 1_000_000.0);
    });
}
