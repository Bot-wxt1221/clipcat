use async_trait::async_trait;
use clipcat::{ClipEntry, ClipboardMode};
use clipcat_proto as proto;
use tonic::Request;

use crate::{
    error::{
        BatchRemoveClipError, ClearClipError, GetClipError, GetCurrentClipError, GetLengthError,
        InsertClipError, ListClipError, MarkClipError, RemoveClipError, UpdateClipError,
    },
    Client,
};

#[async_trait]
pub trait Manager {
    async fn get(&self, id: u64) -> Result<ClipEntry, GetClipError>;

    async fn get_current_clip(&self, mode: ClipboardMode)
        -> Result<ClipEntry, GetCurrentClipError>;

    async fn update(
        &self,
        id: u64,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<(bool, u64), UpdateClipError>;

    async fn mark(&self, id: u64, mode: ClipboardMode) -> Result<bool, MarkClipError>;

    async fn insert(
        &self,
        data: &[u8],
        mime: mime::Mime,
        clipboard_mode: ClipboardMode,
    ) -> Result<u64, InsertClipError>;

    async fn insert_clipboard(
        &self,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<u64, InsertClipError> {
        self.insert(data, mime, ClipboardMode::Clipboard).await
    }

    async fn insert_primary(&self, data: &[u8], mime: mime::Mime) -> Result<u64, InsertClipError> {
        self.insert(data, mime, ClipboardMode::Selection).await
    }

    async fn length(&self) -> Result<usize, GetLengthError>;

    async fn list(&self) -> Result<Vec<ClipEntry>, ListClipError>;

    async fn remove(&self, id: u64) -> Result<bool, RemoveClipError>;

    async fn batch_remove(&self, ids: &[u64]) -> Result<Vec<u64>, BatchRemoveClipError>;

    async fn clear(&self) -> Result<(), ClearClipError>;
}

#[async_trait]
impl Manager for Client {
    async fn get(&self, id: u64) -> Result<ClipEntry, GetClipError> {
        proto::ManagerClient::new(self.channel.clone())
            .get(Request::new(proto::GetRequest { id }))
            .await
            .map_err(|source| GetClipError::Status { source, id })?
            .into_inner()
            .data
            .map_or_else(|| Err(GetClipError::Empty), |data| Ok(data.into()))
    }

    async fn get_current_clip(
        &self,
        mode: ClipboardMode,
    ) -> Result<ClipEntry, GetCurrentClipError> {
        proto::ManagerClient::new(self.channel.clone())
            .get_current_clip(Request::new(proto::GetCurrentClipRequest { mode: mode.into() }))
            .await
            .map_err(|source| GetCurrentClipError::Status { source, mode })?
            .into_inner()
            .data
            .map_or_else(|| Err(GetCurrentClipError::Empty), |data| Ok(data.into()))
    }

    async fn update(
        &self,
        id: u64,
        data: &[u8],
        mime: mime::Mime,
    ) -> Result<(bool, u64), UpdateClipError> {
        let proto::UpdateResponse { ok, new_id } = proto::ManagerClient::new(self.channel.clone())
            .update(Request::new(proto::UpdateRequest {
                id,
                data: data.to_owned(),
                mime: mime.essence_str().to_owned(),
            }))
            .await
            .map_err(|source| UpdateClipError::Status { source })?
            .into_inner();
        Ok((ok, new_id))
    }

    async fn mark(&self, id: u64, mode: ClipboardMode) -> Result<bool, MarkClipError> {
        let proto::MarkResponse { ok } = proto::ManagerClient::new(self.channel.clone())
            .mark(Request::new(proto::MarkRequest { id, mode: mode.into() }))
            .await
            .map_err(|source| MarkClipError::Status { source, id, mode })?
            .into_inner();
        Ok(ok)
    }

    async fn insert(
        &self,
        data: &[u8],
        mime: mime::Mime,
        clipboard_mode: ClipboardMode,
    ) -> Result<u64, InsertClipError> {
        let proto::InsertResponse { id } = proto::ManagerClient::new(self.channel.clone())
            .insert(Request::new(proto::InsertRequest {
                mode: clipboard_mode.into(),
                data: data.to_owned(),
                mime: mime.essence_str().to_owned(),
            }))
            .await
            .map_err(|source| InsertClipError::Status { source })?
            .into_inner();
        Ok(id)
    }

    async fn length(&self) -> Result<usize, GetLengthError> {
        let proto::LengthResponse { length } = proto::ManagerClient::new(self.channel.clone())
            .length(Request::new(proto::LengthRequest {}))
            .await
            .map_err(|source| GetLengthError::Status { source })?
            .into_inner();
        Ok(usize::try_from(length).unwrap_or(0))
    }

    async fn list(&self) -> Result<Vec<ClipEntry>, ListClipError> {
        let mut list: Vec<_> = proto::ManagerClient::new(self.channel.clone())
            .list(Request::new(proto::ListRequest {}))
            .await
            .map_err(|source| ListClipError::Status { source })?
            .into_inner()
            .data
            .into_iter()
            .map(ClipEntry::from)
            .collect();
        list.sort_unstable();
        Ok(list)
    }

    async fn remove(&self, id: u64) -> Result<bool, RemoveClipError> {
        let proto::RemoveResponse { ok } = proto::ManagerClient::new(self.channel.clone())
            .remove(Request::new(proto::RemoveRequest { id }))
            .await
            .map_err(|source| RemoveClipError::Status { source })?
            .into_inner();
        Ok(ok)
    }

    async fn batch_remove(&self, ids: &[u64]) -> Result<Vec<u64>, BatchRemoveClipError> {
        let proto::BatchRemoveResponse { ids } = proto::ManagerClient::new(self.channel.clone())
            .batch_remove(Request::new(proto::BatchRemoveRequest { ids: Vec::from(ids) }))
            .await
            .map_err(|source| BatchRemoveClipError::Status { source })?
            .into_inner();
        Ok(ids)
    }

    async fn clear(&self) -> Result<(), ClearClipError> {
        proto::ManagerClient::new(self.channel.clone())
            .clear(Request::new(proto::ClearRequest {}))
            .await
            .map(|_| ())
            .map_err(|source| ClearClipError::Status { source })
    }
}
