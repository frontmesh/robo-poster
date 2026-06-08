module Api exposing (..)

import Http
import Json.Decode as Decode
import Json.Encode as Encode
import Types exposing (..)


authHeader : Maybe String -> List Http.Header
authHeader token =
    case token of
        Just t ->
            [ Http.header "Authorization" ("Bearer " ++ t) ]

        Nothing ->
            []


loginRequest : String -> String -> String -> (Result Http.Error String -> msg) -> Cmd msg
loginRequest email password apiBaseUrl toMsg =
    Http.post
        { url = apiBaseUrl ++ "/auth/login"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "email", Encode.string email )
                    , ( "password", Encode.string password )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "token" Decode.string)
        }


registerRequest : String -> String -> String -> (Result Http.Error String -> msg) -> Cmd msg
registerRequest email password apiBaseUrl toMsg =
    Http.post
        { url = apiBaseUrl ++ "/auth/register"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "email", Encode.string email )
                    , ( "password", Encode.string password )
                    ]
                )
        , expect = Http.expectJson toMsg (Decode.field "token" Decode.string)
        }


getPosts : Maybe String -> String -> (Result Http.Error (List Post) -> msg) -> Cmd msg
getPosts token apiBaseUrl toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/posts"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list postDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


getAccounts : Maybe String -> String -> (Result Http.Error (List Account) -> msg) -> Cmd msg
getAccounts token apiBaseUrl toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/accounts"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list accountDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


getCalendar : Maybe String -> String -> (Result Http.Error (List CalendarDay) -> msg) -> Cmd msg
getCalendar token apiBaseUrl toMsg =
    Http.request
        { method = "GET"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/calendar"
        , body = Http.emptyBody
        , expect = Http.expectJson toMsg (Decode.list calendarDayDecoder)
        , timeout = Nothing
        , tracker = Nothing
        }


createPost : Maybe String -> String -> String -> String -> Maybe String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
createPost token accountId content platform scheduledAt apiBaseUrl toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/posts"
        , body =
            Http.jsonBody
                (Encode.object
                    [ ( "account_id", Encode.string accountId )
                    , ( "content", Encode.string content )
                    , ( "platform", Encode.string platform )
                    , ( "scheduled_at"
                      , case scheduledAt of
                            Just s ->
                                Encode.string s

                            Nothing ->
                                Encode.null
                      )
                    ]
                )
        , expect = Http.expectJson toMsg postDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


publishPost : Maybe String -> String -> String -> (Result Http.Error Post -> msg) -> Cmd msg
publishPost token postId apiBaseUrl toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/posts/" ++ postId ++ "/publish"
        , body = Http.jsonBody Encode.null
        , expect = Http.expectJson toMsg postDecoder
        , timeout = Nothing
        , tracker = Nothing
        }


deletePost : Maybe String -> String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deletePost token postId apiBaseUrl toMsg =
    Http.request
        { method = "DELETE"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/posts/" ++ postId
        , body = Http.emptyBody
        , expect = Http.expectWhatever (\_ -> toMsg (Ok ()))
        , timeout = Nothing
        , tracker = Nothing
        }


generateContent : Maybe String -> String -> String -> String -> (Result Http.Error String -> msg) -> Cmd msg
generateContent token prompt platform apiBaseUrl toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/ai/generate"
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


connectAccount : Maybe String -> String -> (Result Http.Error String -> msg) -> Cmd msg
connectAccount token apiBaseUrl toMsg =
    Http.request
        { method = "POST"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/accounts/connect"
        , body = Http.jsonBody Encode.null
        , expect = Http.expectJson toMsg (Decode.field "url" Decode.string)
        , timeout = Nothing
        , tracker = Nothing
        }


deleteAccount : Maybe String -> String -> String -> (Result Http.Error () -> msg) -> Cmd msg
deleteAccount token accountId apiBaseUrl toMsg =
    Http.request
        { method = "DELETE"
        , headers = authHeader token
        , url = apiBaseUrl ++ "/accounts/" ++ accountId
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
