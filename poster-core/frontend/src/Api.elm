module Api exposing (..)

import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Types exposing (..)


baseUrl : String
baseUrl =
    "http://localhost:3000/api"


loginRequest : String -> String -> (Result Http.Error String -> msg) -> Cmd msg
loginRequest email password toMsg =
    Http.post
        { url = baseUrl ++ "/auth/login"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "email", Encode.string email )
                    , ( "password", Encode.string password )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "token" Decode.string)
        }


registerRequest : String -> String -> (Result Http.Error String -> msg) -> Cmd msg
registerRequest email password toMsg =
    Http.post
        { url = baseUrl ++ "/auth/register"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "email", Encode.string email )
                    , ( "password", Encode.string password )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "token" Decode.string)
        }


getPosts : (Result Http.Error (List Post) -> msg) -> Cmd msg
getPosts toMsg =
    Http.get
        { url = baseUrl ++ "/posts"
        , expect = Http.expectJson toMsg (Decode.list postDecoder)
        }


getAccounts : (Result Http.Error (List Account) -> msg) -> Cmd msg
getAccounts toMsg =
    Http.get
        { url = baseUrl ++ "/accounts"
        , expect = Http.expectJson toMsg (Decode.list accountDecoder)
        }


getCalendar : (Result Http.Error (List CalendarDay) -> msg) -> Cmd msg
getCalendar toMsg =
    Http.get
        { url = baseUrl ++ "/calendar"
        , expect = Http.expectJson toMsg (Decode.list calendarDayDecoder)
        }


createPost : String -> String -> Maybe String -> Maybe String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
createPost token accountId content scheduledAt platform toMsg =
    Http.post
        { url = baseUrl ++ "/posts"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "account_id", Encode.string accountId )
                    , ( "content", Encode.string content )
                    , ( "scheduled_at"
                      , case scheduledAt of
                            Just s ->
                                Encode.string s

                            Nothing ->
                                Encode.null
                      )
                    , ( "platform", Encode.string platform )
                    ]
                )
        , expect = Http.expectJson toMsg postDecoder
        }


publishPost : String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
publishPost token postId toMsg =
    Http.post
        { url = baseUrl ++ "/posts/" ++ postId ++ "/publish"
        , body = Http.jsonBody Encode.null
        , expect = Http.expectJson toMsg postDecoder
        }


generateContent : String -> String -> String -> (Result Http.Error String -> msg) -> Cmd msg
generateContent token prompt platform toMsg =
    Http.post
        { url = baseUrl ++ "/ai/generate"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "prompt", Encode.string prompt )
                    , ( "platform", Encode.string platform )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "content" Decode.string)
        }


connectAccount : (Result Http.Error String -> msg) -> Cmd msg
connectAccount toMsg =
    Http.get
        { url = baseUrl ++ "/accounts/connect"
        , expect = Http.expectJson toMsg (Decode.field "url" Decode.string)
        }


postDecoder : Decode.Decoder Post
postDecoder =
    Decode.map7 Post
        (Decode.field "id" Decode.string)
        (Decode.field "content" Decode.string)
        (Decode.field "mediaUrl" (Decode.nullable Decode.string))
        (Decode.field "scheduledAt" (Decode.nullable Decode.string))
        (Decode.field "publishedAt" (Decode.nullable Decode.string))
        (Decode.field "status" Decode.string)
        (Decode.field "platform" Decode.string)


accountDecoder : Decode.Decoder Account
accountDecoder =
    Decode.map3 Account
        (Decode.field "id" Decode.string)
        (Decode.field "provider" Decode.string)
        (Decode.field "username" Decode.string)


calendarDayDecoder : Decode.Decoder CalendarDay
calendarDayDecoder =
    Decode.map2 CalendarDay
        (Decode.field "date" Decode.string)
        (Decode.field "posts" (Decode.list postDecoder))
