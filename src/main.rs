use teloxide::{dispatching::dialogue::InMemStorage, prelude::*};

type MyDialogue = Dialogue<State, InMemStorage<State>>;
type HandlerResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[derive(Clone, Default)]
pub enum State {
    #[default]
    Start,
    ReceiveInitial,
    ReceivePersent {
        initial: i32,
    },
    ReceiveTime {
        initial: i32,
        persent:i32
    },
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    log::info!("Starting dialogue bot...");

    let bot = Bot::from_env();

    Dispatcher::builder(
        bot,
        Update::filter_message()
            .enter_dialogue::<Message, InMemStorage<State>, State>()
            .branch(dptree::case![State::Start].endpoint(start))
            .branch(dptree::case![State::ReceiveInitial].endpoint(receive_initial))
            .branch(dptree::case![State::ReceivePersent { initial }].endpoint(receive_persent))
            .branch(
                dptree::case![State::ReceiveTime { initial, persent }].endpoint(receive_time),
            ),
    )
    .dependencies(dptree::deps![InMemStorage::<State>::new()])
    .enable_ctrlc_handler()
    .build()
    .dispatch()
    .await;
}

async fn start(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    bot.send_message(msg.chat.id, "Привет! Это бот для подсчета сложных процентов. Напишите начальную сумму вклада.").await?;
    dialogue.update(State::ReceiveInitial).await?;
    Ok(())
}

async fn receive_initial(bot: Bot, dialogue: MyDialogue, msg: Message) -> HandlerResult {
    match msg.text() {
        Some(text) => {
            bot.send_message(msg.chat.id, "Теперь надо узнать процент под который вы внесли сумму.").await?;
            dialogue.update(State::ReceivePersent { initial: text.parse::<i32>().unwrap()}).await?;
        }
        None => {
            bot.send_message(msg.chat.id, "Send me plain text.").await?;
        }
    }

    Ok(())
}

async fn receive_persent(
    bot: Bot,
    dialogue: MyDialogue,
    initial: i32, // Available from `State::ReceiveAge`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(persent)) => {
            bot.send_message(msg.chat.id, "На какой срок вы внесли деньги? Укажите в месяцах.").await?;
            dialogue.update(State::ReceiveTime { initial, persent }).await?;
        }
        _ => {
            bot.send_message(msg.chat.id, "Пришлите цифру.").await?;
        }
    }

    Ok(())
}

async fn receive_time(
    bot: Bot,
    dialogue: MyDialogue,
    (initial, persent): (i32,i32), // Available from `State::ReceiveLocation`.
    msg: Message,
) -> HandlerResult {
    match msg.text().map(|text| text.parse::<i32>()) {
        Some(Ok(time)) => {
            let perc:f32 = (persent as f32/100 as f32) as f32;
            let result = initial as f32*((1 as f32 + perc).powf(time as f32));
            let report = format!("Сумма вклада: {initial}\nПроцент начисляймый банком: {persent}%\nПериодов начисления процентнов: {time}\nВ конце срока у вас будет:{result}");
            bot.send_message(msg.chat.id, report).await?;
            dialogue.exit().await?;
            
        }
        _ => {
            bot.send_message(msg.chat.id, "Send me a number.").await?;
        }
    }

    Ok(())
}