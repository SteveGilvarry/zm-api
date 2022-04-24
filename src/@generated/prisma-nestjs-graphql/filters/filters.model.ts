import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { ID } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class Filters {

    @Field(() => ID, {nullable:false})
    Id!: number;

    @Field(() => String, {nullable:false,defaultValue:''})
    Name!: string;

    @Field(() => Int, {nullable:true})
    UserId!: number | null;

    @Field(() => String, {nullable:false})
    Query_json!: string;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoArchive!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoUnarchive!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoVideo!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoUpload!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoEmail!: number;

    @Field(() => String, {nullable:true})
    EmailTo!: string | null;

    @Field(() => String, {nullable:true})
    EmailSubject!: string | null;

    @Field(() => String, {nullable:true})
    EmailBody!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoMessage!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoExecute!: number;

    @Field(() => String, {nullable:true})
    AutoExecuteCmd!: string | null;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoDelete!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoMove!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoCopy!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoCopyTo!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    AutoMoveTo!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    UpdateDiskSpace!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Background!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    Concurrent!: number;

    @Field(() => Int, {nullable:false,defaultValue:0})
    LockRows!: number;
}
