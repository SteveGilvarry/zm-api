import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereUniqueInput } from './servers-where-unique.input';
import { Type } from 'class-transformer';
import { ServersCreateInput } from './servers-create.input';
import { ServersUpdateInput } from './servers-update.input';

@ArgsType()
export class UpsertOneServersArgs {

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    @Type(() => ServersWhereUniqueInput)
    where!: ServersWhereUniqueInput;

    @Field(() => ServersCreateInput, {nullable:false})
    @Type(() => ServersCreateInput)
    create!: ServersCreateInput;

    @Field(() => ServersUpdateInput, {nullable:false})
    @Type(() => ServersUpdateInput)
    update!: ServersUpdateInput;
}
