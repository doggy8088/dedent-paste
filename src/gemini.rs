use std::error::Error;

use dedent_paste::{
    ClipboardImage, GeminiError, GeminiSettings, build_generate_content_request, check_image_size,
    extract_text_from_response,
};

const RESPONSE_BODY_LIMIT: u64 = 10 * 1024 * 1024;

pub fn generate_text_from_image(
    settings: &GeminiSettings,
    image: &ClipboardImage,
) -> Result<String, Box<dyn Error>> {
    check_image_size(
        image.data.len(),
        settings.system_prompt.len() + settings.user_instruction.len(),
    )?;

    let body =
        build_generate_content_request(&settings.system_prompt, &settings.user_instruction, image);
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
        settings.model
    );

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .timeout_global(Some(settings.timeout))
        .http_status_as_error(false)
        .build()
        .into();

    let mut response = agent
        .post(&url)
        .header("x-goog-api-key", &settings.api_key)
        .header("Content-Type", "application/json")
        .send(&body)?;

    let status = response.status().as_u16();
    let response_body = response
        .body_mut()
        .with_config()
        .limit(RESPONSE_BODY_LIMIT)
        .read_to_string()?;

    match extract_text_from_response(&response_body) {
        Err(GeminiError::InvalidResponse) if !(200..300).contains(&status) => {
            Err(GeminiError::Api {
                status,
                message: "unexpected non-JSON response".to_string(),
            }
            .into())
        }
        result => Ok(result?),
    }
}
