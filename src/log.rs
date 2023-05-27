use tracing_subscriber::{
    fmt::{self, writer::MakeWriterExt},
    prelude::__tracing_subscriber_SubscriberExt,
    Registry,
};

//  pub fn get_subscriber(log_directory: String) {
//     let file_appender = tracing_appender::rolling::daily(log_directory, "shapeless-blog.log");
//     let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

//     let subscriber = Registry::default()
//         .with(
//             fmt::Layer::default()
//                 .with_writer(file_writer.with_max_level(tracing::Level::WARN))
//                 .with_ansi(false),
//         )
//         .with(
//             fmt::Layer::default().with_writer(std::io::stdout.with_max_level(tracing::Level::INFO)),
//         );

//     tracing::subscriber::set_global_default(subscriber).expect("unable to set global subscriber");
// }
