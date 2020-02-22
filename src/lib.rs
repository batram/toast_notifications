#![feature(vec_remove_item)]
use std::ptr::null_mut;
use winrt::windows::data::xml::dom::*;
use winrt::windows::foundation::*;
use winrt::windows::ui::notifications::*;
use winrt::FastHString;
use state::Storage;
use std::sync::RwLock;
static OPEN_NOTIFICATIONS: Storage<RwLock<Vec<String>>> = Storage::new();

type ActionCallback = fn(arguments: &str);

pub enum Void {}
pub type VoidPtr = *mut Void;
pub type Pwstr = *const u16;

#[link(name = "Wevtapi")]
extern "system" {
    pub fn MessageBoxW(hWnd: VoidPtr, lpText: Pwstr, lpCaption: Pwstr, uType: u32) -> i32;
}

pub fn show_deduped_message(notification_id: &String, template: &str, action_callback: ActionCallback, limit: usize) {
    if !have_open_notification(notification_id) {
        if open_notifications_len() < limit {
            add_open_notification(notification_id);

            
            show_toast_message(template, action_callback);
        } else {
            //TODO: Build waitlist with dedup..
        }
    }
}

pub fn show_toast_message(template: &str, action_callback: ActionCallback) {
    let toast_xml = ToastNotificationManager::get_template_content(ToastTemplateType::ToastText02)
        .unwrap()
        .unwrap();

    toast_xml
        .query_interface::<IXmlDocumentIO>()
        .unwrap()
        .load_xml(&FastHString::new(template))
        .unwrap();

    // Create the toast and attach event listeners
    let toast = ToastNotification::create_toast_notification(&toast_xml).unwrap();

    let dismissed_handler =
        TypedEventHandler::new(move |_sender, args: *mut ToastDismissedEventArgs| {
            unsafe {
                println!("event dismmissed! {:?} {:?}", args, (*args).get_reason());
            }

            Ok(())
        });

    match toast.add_dismissed(&*dismissed_handler) {
        Ok(_) => {}
        Err(_) => println!("couldn't attach dismissed_handler"),
    }

    let failed_handler = TypedEventHandler::new(move |_sender, args: *mut ToastFailedEventArgs| {
        unsafe {
            println!("event failed! {:?} {:?}", args, (*args).get_error_code());
        }
        Ok(())
    });

    match toast.add_failed(&*failed_handler) {
        Ok(_) => {}
        Err(_) => println!("couldn't attach failed_handler"),
    }

    let activated_handler = TypedEventHandler::<_, winrt::IInspectable>::new(
        move |_sender, object: *mut winrt::IInspectable| {
            unsafe {
                let iid = format!("{:?}", (*object).get_iids().first().unwrap());
                if "E3BF92F3-C197-436F-8265-0625824F8DAC" == iid {
                    let args = &*std::mem::transmute::<
                        *mut winrt::IInspectable,
                        *mut ToastActivatedEventArgs,
                    >(object);
                    let action_agrument = args.get_arguments().unwrap().to_string();
                    action_callback(action_agrument.as_str());
                } else {
                    unreachable!("Expected ToastActivatedEventArgs iid got: {}", iid);
                }
            };

            Ok(())
        },
    );

    match toast.add_activated(&*activated_handler) {
        Ok(_) => {}
        Err(_) => println!("couldn't attach activated_handler"),
    }

    let toasty_manager =
        ToastNotificationManager::create_toast_notifier_with_id(&FastHString::new(
            "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\\WindowsPowerShell\\v1.0\\powershell.exe",
        ))
        .unwrap()
        .unwrap();

    match toasty_manager.show(&toast) {
        Ok(_) => {}
        Err(e) => {
            show_message(
                "error".to_string(),
                format!("something went wrong : {:?}", e).to_string(),
            );
        }
    }
}

pub fn show_message(msg: String, title: String) -> i32 {
    let mut l_msg: Vec<u16> = msg.encode_utf16().collect();
    let mut l_title: Vec<u16> = title.encode_utf16().collect();
    l_msg.push(0);
    l_title.push(0);

    return unsafe {
        MessageBoxW(
            null_mut(),
            l_msg.as_ptr(),
            l_title.as_ptr(),
            3 | 64, //winuser::MB_YESNOCANCEL | winuser::MB_ICONINFORMATION,
        )
    };
}

pub fn add_open_notification(notification_id: &String) {
    let state_note_list = OPEN_NOTIFICATIONS.get();
    return match state_note_list.try_write() {
        Ok(mut note_list) => {
            note_list.push(notification_id.clone());
        }
        Err(_) => (),
    };
}

pub fn open_notifications_len() -> usize {
    let state_note_list = OPEN_NOTIFICATIONS.get();
    return match state_note_list.try_read() {
        Ok(note_list) => note_list.len(),
        Err(_) => 0,
    };
}

pub fn finish_notification(notification_id: &String) {
    let state_note_list = OPEN_NOTIFICATIONS.get();
    return match state_note_list.try_write() {
        Ok(mut note_list) => {
            note_list.remove_item(notification_id);
        }
        Err(_) => (),
    };
}

pub fn have_open_notification(notification_id: &String) -> bool {
    let state_note_list = OPEN_NOTIFICATIONS.get_or_set( || RwLock::new(Vec::<String>::new()));
    return match state_note_list.try_read() {
        Ok(note_lis) => note_lis.contains(notification_id),
        Err(_) => false,
    };
}
