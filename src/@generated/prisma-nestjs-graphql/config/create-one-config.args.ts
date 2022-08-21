import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { ConfigCreateInput } from './config-create.input';
import { Type } from 'class-transformer';

@ArgsType()
export class CreateOneConfigArgs {

    @Field(() => ConfigCreateInput, {nullable:false})
    @Type(() => ConfigCreateInput)
    data!: ConfigCreateInput;
}
