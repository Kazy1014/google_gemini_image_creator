use async_trait::async_trait;
use google_gemini_image_creator::application::GenerateImageUseCase;
use google_gemini_image_creator::domain::{
    GeminiModel, GeneratedImage, ImageGenerationError, ImageGenerationRepository,
    ImageGenerationRequest,
};

struct MockRepository {
    should_fail: bool,
}

#[async_trait]
impl ImageGenerationRepository for MockRepository {
    async fn generate_image(
        &self,
        _request: &ImageGenerationRequest,
    ) -> Result<GeneratedImage, ImageGenerationError> {
        if self.should_fail {
            Err(ImageGenerationError::ApiError("Mock error".to_string()))
        } else {
            Ok(GeneratedImage::new(
                vec![1, 2, 3, 4],
                GeminiModel::from("gemini-2.5-flash-image".to_string()),
            ))
        }
    }
}

#[tokio::test]
async fn test_generate_image_use_case_success() {
    let repository = MockRepository { should_fail: false };
    let use_case = GenerateImageUseCase::new(repository);
    let request = ImageGenerationRequest::new("test prompt".to_string());

    let result = use_case.execute(request).await;
    assert!(result.is_ok());
    let image = result.unwrap();
    assert_eq!(image.data, vec![1, 2, 3, 4]);
}

#[tokio::test]
async fn test_generate_image_use_case_validation_error() {
    let repository = MockRepository { should_fail: false };
    let use_case = GenerateImageUseCase::new(repository);
    let request = ImageGenerationRequest::new("".to_string());

    let result = use_case.execute(request).await;
    assert!(result.is_err());
    use google_gemini_image_creator::application::use_cases::generate_image::UseCaseError;
    assert!(matches!(result.unwrap_err(), UseCaseError::Validation(_)));
}

#[tokio::test]
async fn test_generate_image_use_case_repository_error() {
    let repository = MockRepository { should_fail: true };
    let use_case = GenerateImageUseCase::new(repository);
    let request = ImageGenerationRequest::new("test prompt".to_string());

    let result = use_case.execute(request).await;
    assert!(result.is_err());
    use google_gemini_image_creator::application::use_cases::generate_image::UseCaseError;
    assert!(matches!(result.unwrap_err(), UseCaseError::Repository(_)));
}
