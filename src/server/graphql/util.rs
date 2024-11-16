use anyhow::anyhow;

pub fn calculate_start_and_length(
    after: Option<i32>,
    before: Option<i32>,
    first: Option<i32>,
    last: Option<i32>,
    default_page_size: i32,
) -> Result<(usize, usize), anyhow::Error> {
    let (start, length) = match (after, before) {
        (None, None) => {
            if let Some(first) = first {
                (0, first)
            } else if let Some(_last) = last {
                Err(anyhow!("Cannot use last without before"))?
            } else {
                (0, default_page_size)
            }
        }
        (Some(after), None) => {
            if let Some(first) = first {
                (after + 1, first)
            } else if let Some(_last) = last {
                Err(anyhow!("Cannot use last without before"))?
            } else {
                (after + 1, default_page_size)
            }
        }
        (None, Some(before)) => {
            if let Some(first) = first {
                (0, first.min(before))
            } else if let Some(last) = last {
                (0.max(before - last), last.min(before))
            } else {
                (
                    (before - default_page_size).max(0),
                    default_page_size.min(before),
                )
            }
        }
        (Some(after), Some(before)) => {
            if let Some(first) = first {
                (after + 1, first.min(before - after - 1))
            } else if let Some(last) = last {
                ((after + 1).max(before - last), last.min(before - after - 1))
            } else {
                (after + 1, default_page_size.min(before - after - 1))
            }
        }
    };
    Ok((start as usize, length as usize))
}

#[cfg(test)]
mod tests {
    use crate::graphql::util::calculate_start_and_length;

    #[test]
    fn test_calculate_result_indices() -> Result<(), anyhow::Error> {
        // No before or after argument
        assert_eq!(
            calculate_start_and_length(None, None, Some(5), None, 3)?,
            (0, 5)
        );
        assert!(calculate_start_and_length(None, None, None, Some(5), 3).is_err());
        assert_eq!(
            calculate_start_and_length(None, None, None, None, 3)?,
            (0, 3)
        );

        // Only after set
        assert_eq!(
            calculate_start_and_length(Some(5), None, None, None, 3)?,
            (6, 3)
        );
        assert_eq!(
            calculate_start_and_length(Some(7), None, Some(5), None, 3)?,
            (8, 5)
        );
        assert!(calculate_start_and_length(Some(5), None, None, Some(7), 3).is_err());

        // Only before set
        assert_eq!(
            calculate_start_and_length(None, Some(1), None, None, 3)?,
            (0, 1)
        );
        assert_eq!(
            calculate_start_and_length(None, Some(5), None, None, 3)?,
            (2, 3)
        );
        assert_eq!(
            calculate_start_and_length(None, Some(7), Some(5), None, 3)?,
            (0, 5)
        );
        assert_eq!(
            calculate_start_and_length(None, Some(7), None, Some(5), 3)?,
            (2, 5)
        );
        assert_eq!(
            calculate_start_and_length(None, Some(3), None, Some(5), 3)?,
            (0, 3)
        );

        // Both before and after set
        assert_eq!(
            calculate_start_and_length(Some(5), Some(7), Some(13), None, 3)?,
            (6, 1)
        );
        assert_eq!(
            calculate_start_and_length(Some(20), Some(30), Some(13), None, 3)?,
            (21, 9)
        );
        assert_eq!(
            calculate_start_and_length(Some(20), Some(40), Some(13), None, 3)?,
            (21, 13)
        );

        assert_eq!(
            calculate_start_and_length(Some(5), Some(7), None, Some(13), 3)?,
            (6, 1)
        );
        assert_eq!(
            calculate_start_and_length(Some(5), Some(7), None, Some(1), 3)?,
            (6, 1)
        );
        assert_eq!(
            calculate_start_and_length(Some(20), Some(40), None, Some(13), 3)?,
            (27, 13)
        );

        assert_eq!(
            calculate_start_and_length(Some(5), Some(7), None, None, 3)?,
            (6, 1)
        );

        Ok(())
    }
}
