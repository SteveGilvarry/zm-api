import { Field } from '@nestjs/graphql';
import { ObjectType } from '@nestjs/graphql';
import { Int } from '@nestjs/graphql';

@ObjectType()
export class FiltersMaxAggregate {

    @Field(() => Int, {nullable:true})
    Id?: number;

    @Field(() => String, {nullable:true})
    Name?: string;

    @Field(() => Int, {nullable:true})
    UserId?: number;

    @Field(() => String, {nullable:true})
    Query_json?: string;

    @Field(() => Int, {nullable:true})
    AutoArchive?: number;

    @Field(() => Int, {nullable:true})
    AutoUnarchive?: number;

    @Field(() => Int, {nullable:true})
    AutoVideo?: number;

    @Field(() => Int, {nullable:true})
    AutoUpload?: number;

    @Field(() => Int, {nullable:true})
    AutoEmail?: number;

    @Field(() => String, {nullable:true})
    EmailTo?: string;

    @Field(() => String, {nullable:true})
    EmailSubject?: string;

    @Field(() => String, {nullable:true})
    EmailBody?: string;

    @Field(() => Int, {nullable:true})
    AutoMessage?: number;

    @Field(() => Int, {nullable:true})
    AutoExecute?: number;

    @Field(() => String, {nullable:true})
    AutoExecuteCmd?: string;

    @Field(() => Int, {nullable:true})
    AutoDelete?: number;

    @Field(() => Int, {nullable:true})
    AutoMove?: number;

    @Field(() => Int, {nullable:true})
    AutoCopy?: number;

    @Field(() => Int, {nullable:true})
    AutoCopyTo?: number;

    @Field(() => Int, {nullable:true})
    AutoMoveTo?: number;

    @Field(() => Int, {nullable:true})
    UpdateDiskSpace?: number;

    @Field(() => Int, {nullable:true})
    Background?: number;

    @Field(() => Int, {nullable:true})
    Concurrent?: number;

    @Field(() => Int, {nullable:true})
    LockRows?: number;
}
