use crate::auth::models::{AuthResponse, Claims, CreateUserRequest, LoginRequest, SettingsResponse, UpdateSettingsRequest, User, UserResponse};
use crate::shared::error::{AppError, AppResult};
use async_trait::async_trait;
use uuid::Uuid;

use crate::email::{is_configured, send_verification_email};
use crate::categories::service::CategoryService;
