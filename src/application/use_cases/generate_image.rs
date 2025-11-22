use crate::domain::{
    ImageGenerationError, ImageGenerationRepository, ImageGenerationRequest, ValidationError,
};

/// 画像生成ユースケース
pub struct GenerateImageUseCase<R>
where
    R: ImageGenerationRepository,
{
    repository: R,
}

impl<R> GenerateImageUseCase<R>
where
    R: ImageGenerationRepository,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    /// 画像を生成する
    pub async fn execute(
        &self,
        request: ImageGenerationRequest,
    ) -> Result<crate::domain::GeneratedImage, UseCaseError> {
        // バリデーション
        request.validate().map_err(UseCaseError::Validation)?;

        // リポジトリを通じて画像生成
        self.repository
            .generate_image(&request)
            .await
            .map_err(UseCaseError::Repository)
    }
}

/// ユースケースエラー
#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    #[error("Repository error: {0}")]
    Repository(#[from] ImageGenerationError),
}
