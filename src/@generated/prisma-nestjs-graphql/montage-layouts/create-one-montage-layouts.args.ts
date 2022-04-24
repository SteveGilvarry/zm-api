import { Field } from '@nestjs/graphql';
import { ArgsType } from '@nestjs/graphql';
import { MontageLayoutsCreateInput } from './montage-layouts-create.input';

@ArgsType()
export class CreateOneMontageLayoutsArgs {

    @Field(() => MontageLayoutsCreateInput, {nullable:false})
    data!: MontageLayoutsCreateInput;
}
