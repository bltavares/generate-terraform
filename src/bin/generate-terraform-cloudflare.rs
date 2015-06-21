extern crate cloudflare;
extern crate hyper;

use std::env;
use cloudflare::Authentication;
use hyper::client::Client;
use std::io;
use std::fs::File;

fn auth() -> Authentication {
    let email = env::var_os("CLOUDFLARE_EMAIL")
        .expect("CLOUDFLARE_EMAIL is not set")
        .to_str()
        .unwrap()
        .to_owned();
    let token = env::var_os("CLOUDFLARE_TOKEN").expect("CLOUDFLARE_TOKEN is not set")
        .to_str()
        .unwrap()
        .to_owned();
    let domain = env::var_os("CLOUDFLARE_DOMAIN").expect("CLOUDFLARE_DOMAIN is not set")
        .to_str()
        .unwrap()
        .to_owned();

    Authentication {
        email: email,
        token: token,
        domain: Some(domain)
    }
}

trait TfElement {
    fn encode(&self) -> String;
}

impl<E : TfElement> TfElement for Vec<E> {
    fn encode(&self) -> String {
        let children : Vec<String> = self.iter()
            .map(|x| x.encode())
            .collect();
        format!("[{}]",  children.connect(",\n"))
    }
}

impl TfElement for cloudflare::Record {
    fn encode(&self) -> String {
        format!(
            r#"
                "cloudflare_record.{name}": {{
                    "type": "cloudflare_record",
                    "primary": {{
                        "id": "{id}",
                        "attributes": {{
                            "domain": "{domain}",
                            "hostname": "{hostname}",
                            "id": "{id}",
                            "name": "{name}",
                            "priority": "{priority}",
                            "ttl": "{ttl}",
                            "type": "{record_type}",
                            "value": "{value}"
                        }}
                    }}
                }}"#,
            name=self.name,
            id=self.rec_id,
            domain=self.zone_name,
            hostname=self.zone_name,
            priority=self.prio.as_ref().unwrap_or(&"".to_owned()),
            ttl=self.ttl,
            value=self.content,
            record_type=self.record_type)
    }
}

fn main() {
    let file = env::var_os("FILE");
    let mut output : Box<std::io::Write>;

    output = if file.is_some() {
        Box::new(File::open(file.unwrap()).ok().expect("Couldn't open the output file"))
    } else {
        Box::new(io::stdout())
    };

    let auth = auth();
    let mut client = Client::new();
    let records = cloudflare::list_records(&mut client, &auth).ok().expect("Failed to load information from CloudFlare. Try again later");

    output.write("Paste this into your .tfvars under the resources object...\n".as_bytes()).ok().expect("Could not write message to output");
    output.write(records.encode().as_bytes()).ok().expect("Could not write tfvars to output");
}
