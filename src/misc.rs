extern crate serde;
extern crate serde_json;

use template::api::MessageStandard;
use template::api::{chat,im,groups,channels};
use template::api::requests;
use template::api::requests::Client;

use template::plugin_api_v1::Slack;

use self::serde_json::{ser,Value};

use config::CONFIG;

/// Finds out if the message is in a channel there the user are not allowed to post
pub fn delete_post_from_channel(message: &MessageStandard, slack: &Slack) {
    let client = requests::default_client().unwrap();

    // If the messages has a "thread_ts" value, it is not considered a thread and not a post,
    // but if the "thread_ts" value is "None" it is a post.
    if message.thread_ts.is_none() {
        let (channel_id, channel_name) = find_channel_id_and_name(message, slack, &client);

        info!("channel_id: {:?} - channel_name: {:?}", &channel_id, &channel_name);

        if CONFIG.channels.as_ref().unwrap().get(&channel_name).is_some() {
            delete_post(message, &channel_id, &channel_name, slack, &client);
        }
    }
}

/// Finds the name of the channel/group and returns it along with the channel/group ID
/// Note: In slack a private channel is in there api called a group (yes, I know it is confusing,
/// but keep it in mind then reading the comments for this function).
fn find_channel_id_and_name(message: &MessageStandard, slack: &Slack, client: &Client) -> (String, String) {
    let mut channel_id = String::new();
    let mut channel_name = String::new();

    // Check of the message ID is for a channel
    let result = channels::info(client, &slack.api_token, &channels::InfoRequest {
        channel: &message.channel.as_ref().unwrap(),
    });
    match result { // if the ID belongs to a channel, the channel_id and channel_name will be returned
        Ok(c) => {
            channel_id = c.channel.as_ref().unwrap().id.as_ref().unwrap().clone();
            channel_name = c.channel.as_ref().unwrap().name.as_ref().unwrap().clone();
            return (channel_id, format!("#{}", channel_name));
        },
        Err(_) => (),
    }

    // Check of the message ID is for group
    let result = groups::info(client, &slack.api_token, &groups::InfoRequest {
        channel: &message.channel.as_ref().unwrap(),
    });
    match result { // if the ID belongs to a group, the channel_id and group_name will be return
        Ok(c) => {
            channel_id = c.group.as_ref().unwrap().id.as_ref().unwrap().clone();
            channel_name = c.group.as_ref().unwrap().name.as_ref().unwrap().clone();
        },
        Err(_) => (),
    }

    (channel_id, format!("#{}", channel_name))
}

/// Deletes the message and sends a response to the user that his message has been deleted.
fn delete_post(message: &MessageStandard, channel_id: &String, channel_name: &String, slack: &Slack, client: &Client) {
    let result = chat::delete(client, &slack.admin_api_token, &chat::DeleteRequest {
        ts: &message.ts.as_ref().unwrap(),
        channel: &message.channel.as_ref().unwrap(),
        as_user: Some(true),
    });

    debug!("DeleteResult: {:?}", result);
    info!("Deleted message form user '{}' in channel '{}'", message.user.as_ref().unwrap(), message.channel.as_ref().unwrap());

    // Opens a direct message channel between the bot and the user of the deleted message.
    // This is not necessary if there already is a direct message channel between user and the bot,
    // but instead of checking/find an already existing one just run this command and it will give you one.
    let result = im::open(client, &slack.api_token, &im::OpenRequest {
        user: message.user.as_ref().unwrap(),
        return_im: Some(true),
    });
    let instant_message = result.unwrap().channel.unwrap();

    let result = chat::post_message(client, &slack.api_token, &chat::PostMessageRequest {
        channel: instant_message.id.as_ref().unwrap(),
        text: &format!("You are not allowed to create a post this channel <#{}|{}>, please use the thread function. I have attached the message you was trying to post, Stay PANDA!",
                       &channel_id,
                       &channel_name),
        parse: None,
        link_names: None,
        attachments: Some(&format!(r#"[{{"title": "You message", "text": {}}}]"#, &ser::to_string(&Value::String(message.text.as_ref().unwrap().clone())).unwrap())),
        unfurl_links: None,
        unfurl_media: None,
        username: Some("best-bot"),
        as_user: Some(true),
        icon_url: None,
        icon_emoji: Some("best"),
        thread_ts: None,
        reply_broadcast: None,
    });
    match result {
        Ok(r) => debug!("PostResult: {:?}", r),
        Err(e) => error!("Failed to send message to user. Error: {:?}", e)
    }

}