use std::sync::Arc;

use sys;

use ::{ State, SteamError };
use callback;

const CALLBACK_BASE_ID: i32 = 500;

pub enum LobbyType {
    Private,
    FriendsOnly,
    Public ,
    Invisible,
}

#[derive(Debug)]
pub struct LobbyId(u64);

/// Access to the steam matchmaking interface
pub struct Matchmaking {
    pub(crate) state: Arc<State>,
    pub(crate) inner: *mut sys::ISteamMatchmaking,
}

impl Matchmaking {
    pub fn request_lobby_list<F>(&self, mut cb: F)
        where F: FnMut(Result<Vec<LobbyId>, SteamError>) + Send + Sync + 'static
    {
        unsafe {
            let api_call = sys::SteamAPI_ISteamMatchmaking_RequestLobbyList(self.inner);
            callback::register_call_result::<sys::LobbyMatchList, _>(
                &self.state, api_call, CALLBACK_BASE_ID + 10,
                move |v, io_error| {
                   cb(if io_error {
                      Err(SteamError::IOFailure)
                   } else {
                       let mut out = Vec::with_capacity(v.lobbies_matching as usize);
                       for idx in 0 .. v.lobbies_matching {
                           out.push(LobbyId(sys::SteamAPI_ISteamMatchmaking_GetLobbyByIndex(sys::steam_rust_get_matchmaking(), idx as _)));
                       }
                       Ok(out)
                   })
            });
        }
    }

    pub fn create_lobby<F>(&self, ty: LobbyType, max_members: u32, mut cb: F)
        where F: FnMut(Result<LobbyId, SteamError>) + Send + Sync + 'static 
    {
        unsafe {
            let ty = match ty {
                LobbyType::Private => sys::LobbyType::Private,
                LobbyType::FriendsOnly => sys::LobbyType::FriendsOnly,
                LobbyType::Public => sys::LobbyType::Public,
                LobbyType::Invisible => sys::LobbyType::Invisible,
            };
            let api_call = sys::SteamAPI_ISteamMatchmaking_CreateLobby(self.inner, ty, max_members as _);
            callback::register_call_result::<sys::LobbyCreated, _>(
                &self.state, api_call, CALLBACK_BASE_ID + 13,
                move |v, io_error| {
                    cb(if io_error {
                        Err(SteamError::IOFailure)
                    } else if v.result != sys::SResult::Ok {
                        Err(v.result.into())
                    } else {
                        Ok(LobbyId(v.lobby_steam_id))
                    })
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use ::SteamApi;
    use super::LobbyType;

    #[test]
    fn test_lobby() {
        let api = SteamApi::init().unwrap();
        let mm = api.matchmaking();

        mm.request_lobby_list(|v| {
            println!("List: {:?}", v);
        });
        
        mm.create_lobby(LobbyType::Private, 4, |v| {
            println!("Create: {:?}", v);
        });

        for _ in 0 .. 100 {
            api.run_callbacks();
            ::std::thread::sleep(::std::time::Duration::from_millis(100));
        }
    }
}