import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersUpdateInput } from './servers-update.input';
import { Type } from 'class-transformer';
import { ServersWhereUniqueInput } from './servers-where-unique.input';

@ArgsType()
export class UpdateOneServersArgs {

    @Field(() => ServersUpdateInput, {nullable:false})
    @Type(() => ServersUpdateInput)
    data!: ServersUpdateInput;

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    @Type(() => ServersWhereUniqueInput)
    where!: ServersWhereUniqueInput;
}
