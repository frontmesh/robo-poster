module Api exposing (..)

import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Types exposing (..)


baseUrl : String
baseUrl =
    "http://localhost:3000/api"


authHeader : Maybe String -> List Http.header
authHeader token =
    case token of
        Just t ->
            [ Http.header "Authorization" ("Bearer " ++ t) ]

        Nothing ->
            []


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


getPosts : Maybe String -> (Result Http.Error (List Post) -> msg) -> Cmd msg
getPosts token toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = baseUrl ++ "/posts"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list postDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


getAccounts : Maybe String -> (Result Http.Error (List Account) -> msg) -> Cmd msg
getAccounts token toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = baseUrl ++ "/accounts"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list accountDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


getCalendar : Maybe String -> (Result Http.Error (List CalendarDay) -> msg) -> Cmd msg
getCalendar token toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = baseUrl ++ "/calendar"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list calendarDayDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


createPost : Maybe String -> String -> String -> Maybe String -> Maybe String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
createPost token accountId content scheduledAt platform toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = baseUrl ++ "/posts"
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
        , timeout = Nothing
        , tracker = Nothing
        }


publishPost : Maybe String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
publishPost token postId toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = baseUrl ++ "/posts/" ++ postId ++ "/publish"
        , body = Http.jsonBody Encode.null
        , expect = Http.expectJson toMsg postDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


generateContent : Maybe String -> String -> String -> (Result Http.Error String -> msg) -> Cmd msg
generateContent token prompt platform toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = baseUrl ++ "/ai/generate"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "prompt", Encode.string prompt )
                    , ( "platform", Encode.string platform )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "content" Decode.string)
        , timeout = Nothing
        , tracker = Nothing
        }


connectAccount : Maybe String -> (Result Http.Error String -> msg) -> Cmd msg
connectAccount token toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = baseUrl ++ "/accounts/connect"
        , body = Http.jsonBody Encode.null
        , expect = Http.expectJson toMsg (Decode.field "url" Decode.string)
        , timeout = Nothing
        , tracker = Nothing
        }


deleteAccount : Maybe String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteAccount token accountId toMsg =
    Http.request
        { method = "DELETE"
        , headers = authHeader token
        , url = baseUrl ++ "/accounts/" ++ accountId
        , body = Http.emptyBody
        , expect = Http.expectWhatever (\_ -> toMsg (Ok ()))
        , timeout = Nothing
        , tracker = Nothing
        }


postDecoder : Decode.Decoder Post
postDecoder =
    Decode.map8 Post
        (Decode.field "id" Decode.string)
        (Decode.field "content" Decode.string)
        (Decode.field "media_url" (Decode.nullable Decode.string))
        (Decode.field "scheduled_at" (Decode.nullable Decode.string))
        (Decode.field "published_at" (Decode.nullable Decode.string))
        (Decode.field "status" Decode.string)
        (Decode.field "platform" Decode.string)
        (Decode.field "account_id" Decode.string)


accountDecoder : Decode.Decoder Account
accountDecoder =
    Decode.map6 Account
        (Decode.field "id" Decode.string)
        (Decode.field "provider" Decode.string)
        (Decode.field "provider_user_id" Decode.string)
        (Decode.field "username" Decode.string)
        (Decode.field "token_expires_at" (Decode.nullable Decode.string))
        (Decode.field "created_at" Decode.string)


calendarDayDecoder : Decode.Decoder CalendarDay
calendarDayDecoder =
    Decode.map2 CalendarDay
        (Decode.field "date" Decode.string)
        (Decode.field "posts" (Decode.list postDecoder))
