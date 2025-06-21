extern crate vrng;

use tokio::{io::{AsyncReadExt, AsyncWriteExt, stdin, stdout},
    runtime as tokio_rt};
use vrng::VecRng;

async fn real_mainlbl()
    -> !
{
    let mut sinbuf = VecRng::<u8>::with_capacity(4096);
    let mut i = stdin();
    let mut o = stdout();
    loop
    {
        let (mut h, mut b) = sinbuf.spare_capacity_mut();
        let mut n_read = i.read_buf(&mut h).await.unwrap();
        if n_read == h.len()
        {
            n_read += i.read_buf(&mut b).await.unwrap();
            /* no reallocation logic :( */
        }
        unsafe { sinbuf.back_init_change(n_read as isize); }
        let (h, b) = sinbuf.as_ref();
        let mut nwritten = o.write(h).await.unwrap();
        if nwritten == h.len()
        {
            nwritten += o.write(b).await.unwrap();
        }
        unsafe { sinbuf.head_init_change(-(nwritten as isize)); }
    }
}

fn main()
{
    let rt = tokio_rt::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(real_mainlbl());
}
