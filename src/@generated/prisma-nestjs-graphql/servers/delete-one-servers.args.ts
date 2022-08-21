import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ServersWhereUniqueInput } from './servers-where-unique.input';
import { Type } from 'class-transformer';

@ArgsType()
export class DeleteOneServersArgs {

    @Field(() => ServersWhereUniqueInput, {nullable:false})
    @Type(() => ServersWhereUniqueInput)
    where!: ServersWhereUniqueInput;
}
