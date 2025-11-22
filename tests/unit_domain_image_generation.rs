use google_gemini_image_creator::domain::image_generation::ImageGenerationError;

#[test]
fn test_image_generation_error_display() {
    let err = ImageGenerationError::AuthenticationError("Invalid API key".to_string());
    assert!(err.to_string().contains("authentication"));
}
