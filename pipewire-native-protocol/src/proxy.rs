pub trait Proxy {
    type Event;
    
    fn id(&self) -> i32;
    fn get_channel(&mut self) -> &mut tokio::sync::mpsc::Receiver<Self::Event>;
    fn get_connection(&self) -> std::sync::Arc<tokio::sync::Mutex<crate::PipewireWriter>>;


    // Send a sync message through the connection with the Id of the current proxy
    // When we receive a done message we route it to the proxy with the id
    // This is not absolute standard pipewire according to spec.
    // But it seems like accepted usage according to the tutorials
    async fn sync(&mut self) -> Result<(), std::io::Error> {
        let con = self.get_connection();
        let mut connection = con.lock().await;
        let seq = connection.seq + 1; 
        connection.call_method(crate::core_proxy::CORE_ID, <crate::core_proxy::Sync as spa::opcode::MessageOpCode>::OP_CODE, crate::core_proxy::Sync {
            id: self.id(),
            seq: seq,
        }).await
    }
}