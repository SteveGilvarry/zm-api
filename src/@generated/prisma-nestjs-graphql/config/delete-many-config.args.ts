import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigWhereInput } from './config-where.input';

@ArgsType()
export class DeleteManyConfigArgs {

    @Field(() => ConfigWhereInput, {nullable:true})
    where?: ConfigWhereInput;
}
