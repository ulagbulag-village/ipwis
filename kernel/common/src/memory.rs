use ipis::{
    async_trait::async_trait,
    bytecheck::CheckBytes,
    core::{
        anyhow::Result,
        signed::{IsSigned, Serializer},
    },
    rkyv::{
        de::deserializers::SharedDeserializeMap, validation::validators::DefaultValidator, Archive,
        Deserialize, Serialize,
    },
};

use crate::data::{ExternData, ExternDataRef};

#[async_trait]
pub trait Memory: Send + Sync {
    fn is_null(&self, data: ExternDataRef) -> Result<bool> {
        Ok((data as *const ExternData).is_null())
    }

    fn host_check(&self, data: ExternData) -> Result<()>;

    fn host_ptr<T>(&self, ptr: ExternDataRef) -> Result<*const T> {
        self.host_check(ExternData {
            ptr,
            // note: platform-specific types may break the code!
            len: ::core::mem::size_of::<T>().try_into()?,
        })
        // safety: checking is already done by `host_check`
        .map(|()| unsafe { self.host_ptr_unchecked(ptr) })
    }

    unsafe fn host_ptr_unchecked<T>(&self, ptr: ExternDataRef) -> *const T;

    fn host_ref<T>(&self, ptr: ExternDataRef) -> Result<&T> {
        self.host_ptr(ptr)
            // safety: checking is already done by `host_ptr`
            .map(|ptr| unsafe { &*ptr })
    }

    fn host_ptr_mut<T>(&mut self, ptr: ExternDataRef) -> Result<*mut T> {
        self.host_check(ExternData {
            ptr,
            // note: platform-specific types may break the code!
            len: ::core::mem::size_of::<T>().try_into()?,
        })
        // safety: checking is already done by `host_check`
        .map(|()| unsafe { self.host_ptr_mut_unchecked(ptr) })
    }

    unsafe fn host_ptr_mut_unchecked<T>(&mut self, ptr: ExternDataRef) -> *mut T;

    fn host_ref_mut<T>(&mut self, ptr: ExternDataRef) -> Result<&mut T> {
        self.host_ptr_mut(ptr)
            // safety: checking is already done by `host_ptr_mut`
            .map(|ptr| unsafe { &mut *ptr })
    }

    fn load(&self, data: ExternDataRef) -> Result<&[u8]> {
        // safety: checking is already done by `host_ptr`
        let data = unsafe { *self.host_ptr(data)? };
        self.load_raw(data)
    }

    fn load_raw(&self, data: ExternData) -> Result<&[u8]> {
        self.host_check(data)
            // safety: checking is already done by `host_check`
            .map(|()| unsafe {
                let ptr = self.host_ptr_unchecked(data.ptr);
                ::core::slice::from_raw_parts(ptr, data.len as usize)
            })
    }

    fn load_mut(&mut self, data: ExternDataRef) -> Result<&mut [u8]> {
        // safety: checking is already done by `host_ptr_mut`
        let data = unsafe { *self.host_ptr_mut(data)? };
        self.load_mut_raw(data)
    }

    fn load_mut_raw(&mut self, data: ExternData) -> Result<&mut [u8]> {
        self.host_check(data)
            // safety: checking is already done by `host_check`
            .map(|()| unsafe {
                let ptr = self.host_ptr_mut_unchecked(data.ptr);
                ::core::slice::from_raw_parts_mut(ptr, data.len as usize)
            })
    }

    async fn dump(&mut self, data: &[u8]) -> Result<ExternData>;

    async fn dump_doubled(&mut self, data: &[u8]) -> Result<ExternData> {
        let data = self.dump(data).await?.as_bytes();
        self.dump(&data).await
    }

    async fn dump_doubled_null(&mut self) -> Result<ExternData> {
        let data = ExternData::default().as_bytes();
        self.dump(&data).await
    }

    async fn dump_doubled_object<T>(&mut self, data: &T) -> Result<ExternData>
    where
        T: Archive + Serialize<Serializer> + IsSigned + Clone + Send + Sync,
        <T as Archive>::Archived:
            for<'a> CheckBytes<DefaultValidator<'a>> + Deserialize<T, SharedDeserializeMap>,
    {
        self.dump_doubled(&data.to_bytes()?).await
    }

    async fn dump_to(&mut self, src: &[u8], dst: ExternDataRef) -> Result<()> {
        // safety: mutability makes this block thread-safe
        let dst = unsafe { ::core::mem::transmute(self.host_ref_mut::<ExternData>(dst)?) };
        self.dump_to_raw(src, dst).await
    }

    async fn dump_to_raw(&mut self, src: &[u8], dst: &mut ExternData) -> Result<()> {
        *dst = self.dump(src).await?;
        Ok(())
    }

    async fn dump_error_to(
        &mut self,
        err: ::ipis::core::anyhow::Error,
        dst: ExternDataRef,
    ) -> Result<()> {
        self.dump_to(&err.to_string().into_bytes(), dst).await
    }

    fn set_len(&mut self, len: ExternDataRef, dst: ExternDataRef) -> Result<()> {
        // safety: mutability makes this block thread-safe
        let dst = unsafe { ::core::mem::transmute(self.host_ref_mut::<ExternData>(dst)?) };
        self.set_len_raw(len, dst);
        Ok(())
    }

    fn set_len_raw(&mut self, len: ExternDataRef, dst: &mut ExternData) {
        dst.len = len
    }
}
