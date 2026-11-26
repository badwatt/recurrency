use crate::api::client::{ApiError, api_delete, api_get, api_post, api_put};
use crate::models::debt::{
    CreateDebtRequest, Debt, DebtPayment, RecordDebtPaymentRequest, UpdateDebtRequest,
};
use uuid::Uuid;

pub async fn list() -> Result<Vec<Debt>, ApiError> {
    api_get("/api/debts").await
}

pub async fn create(req: &CreateDebtRequest) -> Result<Debt, ApiError> {
    api_post("/api/debts", req).await
}

pub async fn update(id: &Uuid, req: &UpdateDebtRequest) -> Result<Debt, ApiError> {
    api_put(&format!("/api/debts/{id}"), req).await
}

pub async fn delete(id: &Uuid) -> Result<(), ApiError> {
    api_delete(&format!("/api/debts/{id}")).await
}

pub async fn record_payment(id: &Uuid, req: &RecordDebtPaymentRequest) -> Result<Debt, ApiError> {
    api_post(&format!("/api/debts/{id}/payments"), req).await
}

pub async fn payments(id: &Uuid) -> Result<Vec<DebtPayment>, ApiError> {
    api_get(&format!("/api/debts/{id}/payments")).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::test_support::{with_response, with_server};
    use chrono::{NaiveDate, Utc};
    use rust_decimal_macros::dec;
    use serial_test::serial;

    fn sample_debt() -> Debt {
        Debt {
            id: Uuid::new_v4(),
            user_id: Uuid::nil(),
            account_id: Uuid::nil(),
            category_id: Uuid::nil(),
            name: "Loan".into(),
            currency: "EUR".into(),
            total_amount: dec!(100),
            remaining_amount: dec!(80),
            paid_amount: dec!(20),
            debt_type: "manual".into(),
            monthly_payment: None,
            payment_days: Vec::new(),
            start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            next_payment_date: None,
            subscription_id: None,
            is_active: true,
            notes: None,
            logo_url: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    #[serial]
    async fn debt_endpoints_exchange_json() {
        let debt = sample_debt();
        let json = serde_json::to_string(&debt).unwrap();
        let listed: Vec<Debt> = with_server(format!("[{json}]"), list()).await.unwrap();
        assert_eq!(listed.len(), 1);

        let created: Debt = with_server(
            json.clone(),
            create(&CreateDebtRequest {
                name: "Loan".into(),
                total_amount: dec!(100),
                currency: "EUR".into(),
                account_id: Uuid::nil(),
                category_id: None,
                debt_type: "manual".into(),
                monthly_payment: None,
                payment_days: Vec::new(),
                first_payment: None,
                clear_first_payment: false,
                start_date: None,
                notes: None,
                logo_url: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(created.id, debt.id);

        let updated: Debt = with_server(
            json.clone(),
            update(&debt.id, &UpdateDebtRequest::default()),
        )
        .await
        .unwrap();
        assert_eq!(updated.name, "Loan");

        let paid: Debt = with_server(
            json.clone(),
            record_payment(
                &debt.id,
                &RecordDebtPaymentRequest {
                    amount: dec!(10),
                    paid_on: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                    note: None,
                },
            ),
        )
        .await
        .unwrap();
        assert_eq!(paid.id, debt.id);

        let payments: Vec<DebtPayment> = with_server(
            format!(
                "[{}]",
                serde_json::to_string(&DebtPayment {
                    id: Uuid::new_v4(),
                    debt_id: debt.id,
                    user_id: Uuid::nil(),
                    amount: dec!(10),
                    paid_on: NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
                    note: None,
                    created_at: Utc::now(),
                })
                .unwrap()
            ),
            payments(&debt.id),
        )
        .await
        .unwrap();
        assert_eq!(payments.len(), 1);

        with_response("204 No Content", "", delete(&debt.id))
            .await
            .unwrap();
    }
}
