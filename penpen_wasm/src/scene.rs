// シーン遷移システム

pub enum Scene {
    Title,
    Playing,
    LevelClear,
    GameOver,
}

pub struct SceneManager {
    pub current: Scene,
}

impl SceneManager {
    pub fn new() -> Self {
        SceneManager { current: Scene::Title }
    }

    pub fn transition_to(&mut self, scene: Scene) {
        self.current = scene;
    }

    pub fn is_title(&self) -> bool {
        matches!(self.current, Scene::Title)
    }

    pub fn is_playing(&self) -> bool {
        matches!(self.current, Scene::Playing)
    }

    pub fn is_level_clear(&self) -> bool {
        matches!(self.current, Scene::LevelClear)
    }

    pub fn is_game_over(&self) -> bool {
        matches!(self.current, Scene::GameOver)
    }

    pub fn as_u8(&self) -> u8 {
        match self.current {
            Scene::Title      => 0,
            Scene::Playing    => 1,
            Scene::LevelClear => 2,
            Scene::GameOver   => 3,
        }
    }
}
