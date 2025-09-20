use crate::prelude::*;

#[derive(Default)]
pub(crate) struct IpInfoProvider {
    options: AppOptions,
    http: HttpClient,
}

impl IpInfoProvider {
    pub fn new(options: AppOptions, http: HttpClient) -> Self {
        Self { options, http }
    }

    async fn get(&self) -> Result<IpInfo, HttpError> {
        let ip_url = Url::parse("https://ipinfo.io").expect("URL should be valid");
        self.http.remove(&ip_url, Some(JSON_EXTENSION)).await;
        self.http.get_json(&ip_url).await
    }

    pub(crate) async fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let info = self
            .get()
            .await
            .map_err(|e| vec![ValidationError::Http(e)])?;
        let mut errors = Vec::new();
        let values = vec![
            ("IP address", self.options.expect_ip.clone(), info.ip),
            (
                "Geolocated country",
                self.options.expect_country.clone(),
                info.country,
            ),
        ];
        for (name, expected, actual) in values {
            let Some(expected) = expected else {
                continue;
            };
            let Err(e) = Validate::expect(&expected, &actual) else {
                continue;
            };
            errors.push(ValidationError::String(name.to_owned(), e));
        }
        errors.to_result()
    }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct IpInfo {
    ip: String,
    hostname: String,
    city: String,
    region: String,
    country: String,
    loc: String,
    org: String,
    postal: String,
    timezone: String,
}

impl Display for IpInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(
            f,
            "{} ({}, {}, {})",
            self.ip, self.city, self.region, self.country
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn validate_env() -> Result<(), ServiceError> {
        // Arrange
        // Act
        let _services = ServiceProvider::create().await?;

        // Assert
        Ok(())
    }

    #[tokio::test]
    async fn validate_none() {
        // Arrange
        let mut ipinfo = IpInfoProvider::default();
        ipinfo.options.expect_ip = None;
        ipinfo.options.expect_country = None;

        // Act
        let result = ipinfo.validate().await;

        // Assert
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn validate_invalid() {
        // Arrange
        let mut ipinfo = IpInfoProvider::default();
        ipinfo.options.expect_ip = Some("203.0.113.1".to_owned());
        ipinfo.options.expect_country = Some("INVALID".to_owned());

        // Act
        let result = ipinfo.validate().await;

        // Assert
        let Err(errors) = result else {
            assert!(false);
            return;
        };
        assert_eq!(errors.len(), 2);
    }
}
