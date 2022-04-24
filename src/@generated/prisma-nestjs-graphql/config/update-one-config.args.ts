import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigUpdateInput } from './config-update.input';
import { ConfigWhereUniqueInput } from './config-where-unique.input';

@ArgsType()
export class UpdateOneConfigArgs {

    @Field(() => ConfigUpdateInput, {nullable:false})
    data!: ConfigUpdateInput;

    @Field(() => ConfigWhereUniqueInput, {nullable:false})
    where!: ConfigWhereUniqueInput;
}
