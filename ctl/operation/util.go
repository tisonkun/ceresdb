/*
 * Licensed to the Apache Software Foundation (ASF) under one
 * or more contributor license agreements.  See the NOTICE file
 * distributed with this work for additional information
 * regarding copyright ownership.  The ASF licenses this file
 * to you under the Apache License, Version 2.0 (the
 * "License"); you may not use this file except in compliance
 * with the License.  You may obtain a copy of the License at
 *
 *   http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing,
 * software distributed under the License is distributed on an
 * "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
 * KIND, either express or implied.  See the License for the
 * specific language governing permissions and limitations
 * under the License.
 */

package operation

import (
	"encoding/json"
	"io"
	"net/http"
	"time"

	"github.com/jedib0t/go-pretty/v6/table"
)

func tableWriter(headers []string) table.Writer {
	header := table.Row{}
	for _, s := range headers {
		header = append(header, s)
	}
	t := table.NewWriter()
	t.AppendHeader(header)
	return t
}

func HttpUtil(method, url string, body io.Reader, response interface{}) error {
	request, _ := http.NewRequest(method, url, body)
	resp, err := (&http.Client{}).Do(request)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	b, err := io.ReadAll(resp.Body)
	if err != nil {
		return err
	}
	err = json.Unmarshal(b, &response)
	return err
}

func FormatTimeMilli(milli int64) string {
	return time.UnixMilli(milli).Format("2006-01-02 15:04:05.000")
}
