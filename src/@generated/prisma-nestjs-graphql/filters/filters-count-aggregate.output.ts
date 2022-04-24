import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class FiltersCountAggregate {

    @Field(() => Int, {nullable:false})
    Id!: number;

    @Field(() => Int, {nullable:false})
    Name!: number;

    @Field(() => Int, {nullable:false})
    UserId!: number;

    @Field(() => Int, {nullable:false})
    Query_json!: number;

    @Field(() => Int, {nullable:false})
    AutoArchive!: number;

    @Field(() => Int, {nullable:false})
    AutoUnarchive!: number;

    @Field(() => Int, {nullable:false})
    AutoVideo!: number;

    @Field(() => Int, {nullable:false})
    AutoUpload!: number;

    @Field(() => Int, {nullable:false})
    AutoEmail!: number;

    @Field(() => Int, {nullable:false})
    EmailTo!: number;

    @Field(() => Int, {nullable:false})
    EmailSubject!: number;

    @Field(() => Int, {nullable:false})
    EmailBody!: number;

    @Field(() => Int, {nullable:false})
    AutoMessage!: number;

    @Field(() => Int, {nullable:false})
    AutoExecute!: number;

    @Field(() => Int, {nullable:false})
    AutoExecuteCmd!: number;

    @Field(() => Int, {nullable:false})
    AutoDelete!: number;

    @Field(() => Int, {nullable:false})
    AutoMove!: number;

    @Field(() => Int, {nullable:false})
    AutoCopy!: number;

    @Field(() => Int, {nullable:false})
    AutoCopyTo!: number;

    @Field(() => Int, {nullable:false})
    AutoMoveTo!: number;

    @Field(() => Int, {nullable:false})
    UpdateDiskSpace!: number;

    @Field(() => Int, {nullable:false})
    Background!: number;

    @Field(() => Int, {nullable:false})
    Concurrent!: number;

    @Field(() => Int, {nullable:false})
    LockRows!: number;

    @Field(() => Int, {nullable:false})
    _all!: number;
}
