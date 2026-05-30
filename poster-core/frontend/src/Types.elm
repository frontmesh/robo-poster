module Types exposing (..)

import Http
import Time


type alias Model =
    { page : Page
    , token : Maybe String
    , userId : Maybe String
    , posts : List Post
    , accounts : List Account
    , calendar : List CalendarDay
    , compose : ComposeModel
    , error : Maybe String
    , loginEmail : String
    , loginPassword : String
    , registerEmail : String
    , registerPassword : String
    , loading : Bool
    }


type Page
    = Login
    | Dashboard
    | Accounts
    | Composer
    | Calendar
    | Analytics
    | Settings


type alias ComposeModel =
    { content : String
    , selectedAccount : Maybe String
    , scheduledAt : Maybe String
    , platform : String
    , aiPrompt : Maybe String
    , aiGenerating : Bool
    }


type alias Post =
    { id : String
    , content : String
    , mediaUrl : Maybe String
    , scheduledAt : Maybe String
    , publishedAt : Maybe String
    , status : String
    , platform : String
    , accountId : String
    }


type alias Account =
    { id : String
    , provider : String
    , providerUserId : String
    , username : String
    , tokenExpiresAt : Maybe String
    , createdAt : String
    }


type alias CalendarDay =
    { date : String
    , posts : List Post
    }


type Msg
    = Navigate Page
    | GotPosts (Result Http.Error (List Post))
    | GotAccounts (Result Http.Error (List Account))
    | GotCalendar (Result Http.Error (List CalendarDay))
    | UpdateComposeContent String
    | UpdateComposePlatform String
    | UpdateComposeAccount String
    | UpdateAiPrompt String
    | GenerateContent
    | GotGeneratedContent (Result Http.Error String)
    | CreatePost
    | PostCreated (Result Http.Error Post)
    | PublishPost String
    | PostPublished (Result Http.Error Post)
    | ConnectAccount
    | GotOAuthUrl (Result Http.Error String)
    | DeleteAccount String
    | AccountDeleted (Result Http.Error ())
    | LoginEmail String
    | LoginPassword String
    | DoLogin
    | LoginResult (Result Http.Error String)
    | RegisterEmail String
    | RegisterPassword String
    | DoRegister
    | RegisterResult (Result Http.Error String)
    | DismissError
    | Logout
